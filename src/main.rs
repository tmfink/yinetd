use std::path::PathBuf;

use clap::Clap;
use log::*;

mod config;
mod error;
mod num;

pub use error::{Error, Result};

const LOG_ENV: &str = "YINETD_LOG";

#[derive(Clap)]
struct Opts {
    /// Path to config file
    #[clap(short = 'c', long = "config")]
    config_path: Option<PathBuf>,

    /// Increase verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    /// Decrease verbosity
    #[clap(short, long, parse(from_occurrences))]
    quiet: i32,

    /// Exit after checking validity of config file
    #[clap(long = "check")]
    check_config: bool,
}

fn init_logging(verbosity: i32) {
    use log::LevelFilter::*;

    let filters: &[LevelFilter] = &[Off, Error, Warn, Info, Debug, Trace];
    let verbosity_idx = num::clamp(verbosity, 0, (filters.len() - 1) as i32) as usize;
    let level = filters[verbosity_idx];

    let mut builder = env_logger::Builder::from_env(LOG_ENV);
    builder.filter_level(level);
    builder.init();
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    init_logging(opts.verbose - opts.quiet + 2);

    let config_path = &opts.config_path.unwrap();
    info!("config: {:?}", &config_path);
    crate::config::parse_config_file(&config_path)?;

    if opts.check_config {
        info!("Exiting after checking config");
        return Ok(());
    }

    // todo(tmfink): start services

    Ok(())
}
