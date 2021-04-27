use std::{
    ffi::OsString,
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

use crate::Result;

/*
config format, based on xinetd.conf(5):

service SERVICE_NAME
{
    ATTRIBUTE ASSIGN_OP VALUE1 VALUE2 ...
    ...
}
 */

pub struct Config {
    pub services: Vec<Service>,
}

#[derive(Debug)]
pub struct Service {
    args: Vec<OsString>,
    port: u16,
}

fn parse_config_str(config: &str) -> Result<Config> {
    unimplemented!()
}

pub fn parse_config_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path: &Path = path.as_ref();
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let file_len = fs::metadata(path)?.len();
    let mut contents = String::with_capacity(file_len as usize);
    buf_reader.read_to_string(&mut contents)?;

    parse_config_str(&contents)
}
