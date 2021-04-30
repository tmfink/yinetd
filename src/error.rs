use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read config")]
    Config(#[from] io::Error),
    #[error("failed to parse config")]
    Parse(#[from] pest::error::Error<crate::config::Rule>),
}

pub type Result<T> = std::result::Result<T, Error>;
