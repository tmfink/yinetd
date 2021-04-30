// pest creates enums in all-caps
#![allow(clippy::upper_case_acronyms)]

use std::{
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

use log::trace;
use pest::{Parser, Token};
use pest_derive::Parser;

use crate::Result;

#[derive(Parser)]
#[grammar = "config_grammar.pest"]
struct ConfigParser;

#[derive(Debug)]
pub struct Config {
    pub services: Vec<Service>,
}

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub args: Vec<String>,
    pub port: u16,
}

fn parse_config_str(config: &str) -> Result<Config> {
    let parser = ConfigParser::parse(Rule::file, config)?;
    for token in parser.tokens() {
        match &token {
            Token::Start { rule, pos } => {
                trace!("found token \"{:?}\" at {:?}", rule, pos);
            }
            Token::End {
                rule: _rule,
                pos: _pos,
            } => {}
        }
    }
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
