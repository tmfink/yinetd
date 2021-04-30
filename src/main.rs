use std::path::{Path, PathBuf};

use clap::Clap;
use log::*;

const LOG_ENV: &str = "YINETD_LOG";

const DEFAULT_CONFIG_PATHS: &[&str] = &["/etc/yinetd.conf"];

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
    let verbosity_idx = yinetd::num::clamp(verbosity, 0, (filters.len() - 1) as i32) as usize;
    let level = filters[verbosity_idx];

    let mut builder = env_logger::Builder::from_env(LOG_ENV);
    builder.filter_level(level);
    builder.init();
}

fn config_path(opts: &Opts) -> anyhow::Result<PathBuf> {
    if let Some(path) = &opts.config_path {
        return Ok(path.clone());
    }
    debug!("No config path passed in, looking in default locations");
    for path in DEFAULT_CONFIG_PATHS.iter() {
        debug!("    looking at {:?}", path);
        let path = Path::new(path);
        if path.exists() {
            return Ok(path.to_path_buf());
        }
    }

    anyhow::bail!(
        "No valid config found. Specify one with `--config` or use one of the defaults: {:?}",
        DEFAULT_CONFIG_PATHS
    );
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    init_logging(opts.verbose - opts.quiet + 2);

    let config_path = config_path(&opts)?;
    info!("config: {:?}", &config_path);
    yinetd::config::parse_config_file(&config_path)?;

    if opts.check_config {
        info!("Exiting after checking config");
        return Ok(());
    }

    // todo(tmfink): start services

    unimplemented!("Starting services not implemented");
}
