// pest creates enums in all-caps
#![allow(clippy::upper_case_acronyms)]

#[cfg(test)]
mod test;

use std::{
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

use log::trace;
use pest::Parser;
use pest_derive::Parser;

use crate::{
    config::Config,
    service::{Service, ServiceOption},
    Error, Result,
};

#[derive(Parser)]
#[grammar = "config_grammar.pest"]
struct ConfigParser;

fn parse_config_str(config: &str) -> Result<Config> {
    let mut parser = ConfigParser::parse(Rule::file, config)?;

    let mut config = Config::new();
    let file_pair = parser.next().unwrap();
    assert_eq!(file_pair.as_rule(), Rule::file);

    let mut default_options = ServiceOption::default();

    for pair in file_pair.into_inner() {
        trace!("pair: {:?}", pair.as_rule());
        match pair.as_rule() {
            Rule::default => {
                let body_pair = pair.into_inner().next().unwrap();
                default_options = ServiceOption::from_service_pair(body_pair)?;
            }
            Rule::service => {
                let mut service_option = ServiceOption::default();
                let mut pair_inner = pair.clone().into_inner();

                let name_pair = pair_inner.next().unwrap();
                assert_eq!(name_pair.as_rule(), Rule::name);
                let service_name = name_pair.as_str();

                if config.has_service(service_name) {
                    return Err(crate::Error::duplicate_service(service_name, &pair));
                }

                let body_pair = pair_inner.next().unwrap();
                service_option.update_from_body_pair(body_pair)?;
                service_option.fill_with_defaults(&default_options);

                let service = Service::from_optioned(service_option, service_name, &pair)?;
                config.add_service(service)?;
            }
            Rule::EOI => {}
            _ => {
                let message = format!("Unexpected pair {:?}", pair.as_rule());
                return Err(crate::error::custom_pest_error(message, pair.as_span()).into());
            }
        }
    }

    Ok(config)
}

pub fn parse_config_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path: &Path = path.as_ref();
    let file = File::open(path).map_err(|err| Error::Config {
        message: "failed to open config".to_string(),
        source: err,
    })?;
    let mut buf_reader = BufReader::new(file);
    let file_len = fs::metadata(path)
        .map_err(|err| Error::Config {
            message: "failed read config file metadata".to_string(),
            source: err,
        })?
        .len();
    let mut contents = String::with_capacity(file_len as usize);
    buf_reader
        .read_to_string(&mut contents)
        .map_err(|err| Error::Config {
            message: "failed to read config file".to_string(),
            source: err,
        })?;

    parse_config_str(&contents).map_err(|err| err.with_path(&path.to_string_lossy()))
}
