mod resources;
mod stacktraces;
mod tracker;
mod view;

use crate::tracker::Tracker;
use anyhow::bail;
use clap::builder::styling::AnsiColor;
use clap::builder::Styles;
use clap::{Parser, Subcommand};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use log::{debug, info};
use std::path::{Path, PathBuf};
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
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    /// Profile
    Profile {
        /// The PID of the Python process to monitor
        pid: u32,
        /// output directory
        output_dir: PathBuf,
        /// ms between samples
        #[arg(short, long)]
        sample_rate: Option<u64>,
        /// capture native stack traces
        #[arg(long)]
        native: bool,
    },
    View {
        /// output directory
        output_dir: PathBuf,
        /// The port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or("RUST_LOG", "py_crude_resource_monitor=info"),
    );

    let args = Args::parse();

    match args.command {
        Subcommands::Profile {
            pid,
            output_dir,
            sample_rate,
            native,
        } => run_profile(pid, output_dir, sample_rate, native),
        Subcommands::View { output_dir , port } => run_view(output_dir, port),
    }
}

fn run_profile(
    pid: u32,
    output_dir: PathBuf,
    sample_rate: Option<u64>,
    native: bool,
) -> anyhow::Result<()> {
    let sample_sleep_duration = Duration::from_millis(sample_rate.unwrap_or(1000));

    std::fs::create_dir_all(&output_dir)?;
    clear_data_dir(&output_dir)?;

    let mut tracker = Tracker::new(pid, output_dir.clone(), native)?;
    while tracker.is_still_tracking() {
        tracker.tick();
        thread::sleep(sample_sleep_duration);
    }

    info!("All processes have exited, exiting");
    info!(
        "View the profile data by running `{} view {:?}`",
        std::env::current_exe()?.to_string_lossy(),
        output_dir.to_string_lossy()
    );

    Ok(())
}

fn clear_data_dir(dir: &Path) -> anyhow::Result<()> {
    let mut files = Vec::new();
    for file in std::fs::read_dir(dir)? {
        let file = file?;
        if file.file_name().to_string_lossy().ends_with(".json") {
            files.push(file);
        }
    }

    let file_names = files
        .iter()
        .map(|f| f.path().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Ok(());
    }

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Are you sure you want to delete {}?",
            file_names.join(", ")
        ))
        .default(false)
        .interact()?;

    if !confirm {
        bail!("User cancelled deletion");
    }

    for file in files {
        debug!("Removing old file {:?}", file.path());
        std::fs::remove_file(file.path())?;
    }

    Ok(())
}

fn run_view(output_dir: PathBuf, port: u16) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(view::run_view(output_dir, port))
}
