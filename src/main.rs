use std::ffi::OsString;

use clap::Clap;
use log::*;

mod num;

const LOG_ENV: &str = "YINETD_LOG";

#[derive(Clap)]
struct Opts {
    #[clap(short = 'c', long = "config")]
    config_path: Option<OsString>,

    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    #[clap(short, long, parse(from_occurrences))]
    quiet: i32,
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

fn main() {
    let opts: Opts = Opts::parse();

    println!("config: {:?}", opts.config_path);

    init_logging(opts.verbose - opts.quiet + 2);
}
