use std::{
    fmt::{Debug, Display},
    io,
    net::IpAddr,
};

use pest::error::Error as PestError;
use pest::error::ErrorVariant as PestErrorVariant;
use pest::{iterators::Pair, Span};
use thiserror::Error;

use crate::{config_types::InetType, parse::Rule};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{message}: {source}")]
    Io {
        message: String,
        #[source]
        source: io::Error,
    },

    #[error("failed to read config")]
    Config {
        message: String,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse config")]
    Parse(#[from] PestError<Rule>),

    #[error("failed to parse value")]
    OptionValueParse {
        option: String,
        #[source]
        context: PestError<Rule>,
    },

    #[error("missing required option {option:?} for service {service:?}")]
    MissingRequiredOption {
        option: String,
        service: String,
        #[source]
        context: PestError<Rule>,
    },

    #[error("duplicate service {service:?}")]
    DuplicateService {
        service: String,
        #[source]
        context: PestError<Rule>,
    },

    #[error("duplicate option {option:?}")]
    DuplicateOption {
        option: String,
        #[source]
        context: PestError<Rule>,
    },

    #[error("expected {expected_type} address, found {addr} for service {service_name:?}")]
    InetVersionAddressMismatch {
        expected_type: InetType,
        addr: IpAddr,
        service_name: String,
    },
}

impl Error {
    pub(crate) fn option_parse<T: Display>(
        option: &str,
        value_pair: &Pair<Rule>,
        source_err: T,
    ) -> Self {
        let message = format!("{}", source_err);
        let context = custom_pest_error(message, value_pair.as_span());
        Self::OptionValueParse {
            option: option.to_string(),
            context,
        }
    }

    pub(crate) fn missing_required_option(
        option: &str,
        service_name: &str,
        service_pair: &Pair<Rule>,
    ) -> Self {
        let message = format!(
            "missing required config option {option:?} for service {service:?}",
            option = option,
            service = service_name
        );
        let context = custom_pest_error(message, service_pair.as_span());
        Self::MissingRequiredOption {
            option: option.to_string(),
            service: service_name.to_string(),
            context,
        }
    }

    pub(crate) fn duplicate_service(service_name: &str, service_pair: &Pair<Rule>) -> Self {
        let message = String::new();
        let context = custom_pest_error(message, service_pair.as_span());
        Self::DuplicateService {
            service: service_name.to_string(),
            context,
        }
    }

    pub(crate) fn duplicate_option(option_name: &str, option_pair: &Pair<Rule>) -> Self {
        let message = String::new();
        let context = custom_pest_error(message, option_pair.as_span());
        Self::DuplicateOption {
            option: option_name.to_string(),
            context,
        }
    }

    /// Add path context to Pest errors
    pub(crate) fn with_path(self, path: &str) -> Self {
        match self {
            Self::Parse(pest_err) => Self::Parse(pest_err.with_path(path)),
            Self::OptionValueParse { option, context } => Self::OptionValueParse {
                option,
                context: context.with_path(path),
            },
            Self::MissingRequiredOption {
                option,
                service,
                context,
            } => Self::MissingRequiredOption {
                option,
                service,
                context: context.with_path(path),
            },
            Self::DuplicateService { service, context } => Self::DuplicateService {
                service,
                context: context.with_path(path),
            },
            Self::DuplicateOption { option, context } => Self::DuplicateOption {
                option,
                context: context.with_path(path),
            },
            _ => self,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn custom_pest_error(message: String, span: Span) -> PestError<Rule> {
    PestError::new_from_span(PestErrorVariant::CustomError { message }, span)
}

pub trait StdIoErrorExt {
    type Into;
    fn with_message(self, message: impl Into<String>) -> Self::Into;
}

impl StdIoErrorExt for std::io::Error {
    type Into = Error;
    fn with_message(self, message: impl Into<String>) -> Self::Into {
        Error::Io {
            message: message.into(),
            source: self,
        }
    }
}

impl<T> StdIoErrorExt for std::io::Result<T> {
    type Into = Result<T>;
    fn with_message(self, message: impl Into<String>) -> Self::Into {
        self.map_err(|err| err.with_message(message))
    }
}
