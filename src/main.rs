mod resources;
mod stacktraces;
mod tracker;

use crate::tracker::Tracker;
use clap::builder::styling::AnsiColor;
use clap::builder::Styles;
use clap::Parser;
use log::{debug, info};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

const CLAP_STYLE: Styles = Styles::styled()
    .header(AnsiColor::Red.on_default().bold())
    .usage(AnsiColor::Red.on_default().bold())
    .literal(AnsiColor::Blue.on_default().bold())
    .placeholder(AnsiColor::Green.on_default());

/// A small utility to monitor resource usage of Python processes
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, styles = CLAP_STYLE)]
struct Args {
    /// The PID of the Python process to monitor
    pid: u32,
    /// output directory
    output_dir: PathBuf,
    /// ms between samples
    sample_rate: Option<u64>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or("RUST_LOG", "py_crude_resource_monitor=info"),
    );

    let args = Args::parse();
    let sample_sleep_duration = Duration::from_millis(args.sample_rate.unwrap_or(1000));

    std::fs::create_dir_all(&args.output_dir)?;
    for file in std::fs::read_dir(&args.output_dir)? {
        let file = file?;
        if file.file_name().to_string_lossy().ends_with(".json") {
            debug!("Removing old file {:?}", file.path());
            std::fs::remove_file(file.path())?;
        }
    }

    let mut tracker = Tracker::new(args.pid, args.output_dir)?;
    while tracker.is_still_tracking() {
        tracker.tick();
        thread::sleep(sample_sleep_duration);
    }

    info!("All processes have exited, exiting");

    Ok(())
}
