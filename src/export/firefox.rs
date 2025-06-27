use crate::types::JsonLine;
use flate2::write::GzEncoder;
use flate2::Compression;
use fxprof_processed_profile::{
    CategoryColor, CpuDelta, Frame, FrameFlags, FrameInfo, GraphColor, Profile, ReferenceTimestamp,
    SamplingInterval, Timestamp,
};
use snafu::{Location, ResultExt, Snafu};
use std::collections::HashMap;
use std::fs::File;
use std::num::ParseIntError;
use std::path::Path;

/// The pid representing the global full-system report.
const GLOBAL_REPORT_PID: u32 = 0;
const MAIN_THREAD_NAME: &str = "MainThread";
const PROCESS_CPU_COUNTER_NAME: &str = "processCPU";
const PROCESS_CPU_CATEGORY_NAME: &str = "CPU";
const PROCESS_CPU_DESCRIPTION: &str = "Process CPU utilization";
const MALLOC_COUNTER_NAME: &str = "malloc";
const MALLOC_CATEGORY_NAME: &str = "Memory";
const MALLOC_DESCRIPTION: &str = "Amount of allocated memory";
const CATEGORY_PYTHON_NAME: &str = "Python";
const CATEGORY_NATIVE_NAME: &str = "Native";

#[derive(Debug, Snafu)]
pub enum ExportError {
    #[snafu(display("Error reading output directory at {location}"))]
    OutputDirRead {
        source: std::io::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Error reading report `{name}` at {location}"))]
    ReadReport {
        source: std::io::Error,
        name: String,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Invalid input file name `{path}` at {location}"))]
    InvalidInputFile {
        source: ParseIntError,
        path: String,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Error deserializing report `{name}` at {location}"))]
    DeserializeReport {
        source: serde_json::Error,
        name: String,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Error serializing reports at {location}"))]
    SerializeReports {
        source: serde_json::Error,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Error writing output file `{path}` at {location}"))]
    WriteOutput {
        source: std::io::Error,
        path: String,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Error writing gzipped output file `{path}` at {location}"))]
    WriteOutputGz {
        source: std::io::Error,
        path: String,
        #[snafu(implicit)]
        location: Location,
    },
}

pub(super) fn export_report(data_dir: &Path, output_path: &Path) -> Result<(), ExportError> {
    let process_to_profile = read_report(data_dir)?;

    let profile = generate_fxprof(process_to_profile);

    write_profile(output_path, profile)?;

    eprintln!(
        "Wrote Firefox profile to {}. Open it in `https://profiler.firefox.com`.",
        output_path.display()
    );

    Ok(())
}

fn read_report(data_dir: &Path) -> Result<HashMap<u32, Vec<JsonLine>>, ExportError> {
    let mut all_processes = HashMap::new();
    for entry in std::fs::read_dir(data_dir).context(OutputDirReadSnafu)? {
        let entry = entry.context(OutputDirReadSnafu)?;
        let name = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let content =
            std::fs::read_to_string(entry.path()).context(ReadReportSnafu { name: &name })?;

        let lines: Vec<JsonLine> = content
            .lines()
            .map(|line| serde_json::from_str(line).context(DeserializeReportSnafu { name: &name }))
            .collect::<Result<_, _>>()?;

        let pid = if name == "global" {
            // Pid 1 is the init process, pid 0 is not real and used as a global placeholder.
            GLOBAL_REPORT_PID
        } else {
            name.parse::<u32>()
                .context(InvalidInputFileSnafu { path: &name })?
        };

        all_processes.insert(pid, lines);
    }

    Ok(all_processes)
}

fn generate_fxprof(processes: HashMap<u32, Vec<JsonLine>>) -> Profile {
    let all_start_times_millis = processes.values().map(|lines| lines[0].time);
    let interval_millis = {
        // todo: inject some additional metadata in the original format
        let lines = processes.values().next().unwrap();
        (lines[1].time - lines[0].time) as u64
    };
    let start_time_millis = all_start_times_millis.min().unwrap();
    let mut profile = Profile::new(
        "python", // todo: inject some additional metadata in the original format
        ReferenceTimestamp::from_millis_since_unix_epoch(start_time_millis as f64),
        SamplingInterval::from_millis(interval_millis),
    );
    let category_python = profile.add_category(CATEGORY_PYTHON_NAME, CategoryColor::Blue);
    let category_native = profile.add_category(CATEGORY_NATIVE_NAME, CategoryColor::Green);

    for (pid, lines) in processes {
        let start_time_millis_process = lines[0].time;
        assert!(start_time_millis_process >= start_time_millis);
        let process = profile.add_process(
            "Process",
            pid,
            Timestamp::from_millis_since_reference(
                (start_time_millis_process - start_time_millis) as f64,
            ),
        );

        let mut all_threads = HashMap::new();
        // adding the main thread first leads to the RAM display corresponding to mainThreadIndex 0 working
        let main_thread = lines
            .iter()
            .flat_map(|line| line.stacktraces.clone())
            .find(|stack_trace| stack_trace.thread_name == Some(MAIN_THREAD_NAME.into()))
            .expect("No main thread found");
        let main_thread_handle = profile.add_thread(
            process,
            main_thread.thread_id as u32,
            Timestamp::from_millis_since_reference(
                (start_time_millis_process - start_time_millis) as f64,
            ),
            true,
        );
        profile.set_thread_name(main_thread_handle, MAIN_THREAD_NAME);
        all_threads.insert(main_thread.thread_id as u32, main_thread_handle);

        let cpu_counter = profile.add_counter(
            process,
            PROCESS_CPU_COUNTER_NAME,
            PROCESS_CPU_CATEGORY_NAME,
            PROCESS_CPU_DESCRIPTION,
        );
        profile.set_counter_color(cpu_counter, GraphColor::Red);
        let memory_counter = profile.add_counter(
            process,
            MALLOC_COUNTER_NAME,
            MALLOC_CATEGORY_NAME,
            MALLOC_DESCRIPTION,
        );
        profile.set_counter_color(cpu_counter, GraphColor::Purple);
        let mut last_cpu = 0.0_f64;
        let mut last_memory = 0.0_f64;

        let mut all_frames = HashMap::new();

        // Initialize the counters with zero to ensure they are relative to zero, not the initial
        // value.
        if let Some(line) = lines.first() {
            let timestamp =
                Timestamp::from_millis_since_reference((line.time - start_time_millis) as f64);
            profile.add_counter_sample(cpu_counter, timestamp, 0., 1);
            profile.add_counter_sample(memory_counter, timestamp, 0., 1);
        }

        for line in lines {
            assert!(line.time >= start_time_millis_process);
            let timestamp =
                Timestamp::from_millis_since_reference((line.time - start_time_millis) as f64);

            for stacktrace in line.stacktraces {
                let thread_id = stacktrace.thread_id as u32;

                let &mut thread = all_threads.entry(thread_id).or_insert_with(|| {
                    profile.add_thread(
                        process,
                        thread_id,
                        Timestamp::from_millis_since_reference(
                            (line.time - start_time_millis) as f64,
                        ),
                        false, // main thread was created above
                    )
                });
                // thread name might not be set in first line of the file
                if let Some(thread_name) = stacktrace.thread_name {
                    profile.set_thread_name(thread, thread_name.as_str());
                }

                let mut stack_frames = Vec::with_capacity(stacktrace.frames.len());
                for frame in stacktrace.frames.iter().rev() {
                    let frame_info = all_frames
                        .entry((frame.filename.clone(), frame.line))
                        .or_insert_with(|| FrameInfo {
                            frame: Frame::Label(
                                profile.intern_string(
                                    format!(
                                        "{} ({}:{})",
                                        frame.name,
                                        frame.short_filename.as_ref().unwrap(),
                                        frame.line
                                    )
                                    .as_str(),
                                ),
                            ),
                            category_pair: if frame.is_entry {
                                category_native.into()
                            } else {
                                category_python.into()
                            },
                            flags: FrameFlags::empty(),
                        });
                    stack_frames.push(frame_info.clone());
                }
                let stack = profile.intern_stack_frames(thread, stack_frames.into_iter());

                let cpu_delta = if thread == main_thread_handle {
                    CpuDelta::from_millis(line.resources.cpu as f64 / 100. * interval_millis as f64)
                } else if let Some(os_thread_id) = stacktrace.os_thread_id {
                    if let Some(resources) = line.resources.thread_resources.get(&os_thread_id) {
                        CpuDelta::from_millis(resources.cpu as f64 / 100. * interval_millis as f64)
                    } else {
                        CpuDelta::ZERO
                    }
                } else {
                    CpuDelta::ZERO
                };
                profile.add_sample(thread, timestamp, stack, cpu_delta, 1);
            }

            let value_delta = line.resources.cpu as f64 - last_cpu;
            last_cpu = line.resources.cpu as f64;
            profile.add_counter_sample(cpu_counter, timestamp, value_delta, 1);

            let value_delta = line.resources.memory as f64 - last_memory;
            last_memory = line.resources.memory as f64;
            profile.add_counter_sample(memory_counter, timestamp, value_delta, 1);
        }
    }

    profile
}

fn write_profile(output_path: &Path, profile: Profile) -> Result<(), ExportError> {
    let output_file = File::create(output_path).context(WriteOutputSnafu {
        path: output_path.display().to_string(),
    })?;

    let mut gz = GzEncoder::new(output_file, Compression::default());

    // Serialize the data to JSON and write it to the gzipped file
    serde_json::to_writer(&mut gz, &profile).context(SerializeReportsSnafu)?;

    // Finish is required to finalize the compressed output and ensure all data is written,
    // without any corruption.
    gz.finish().context(WriteOutputGzSnafu {
        path: output_path.display().to_string(),
    })?;

    Ok(())
}
