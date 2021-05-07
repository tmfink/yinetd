use std::{
    fmt::{Debug, Display},
    io,
};

use pest::error::Error as PestError;
use pest::error::ErrorVariant as PestErrorVariant;
use pest::{iterators::Pair, Span};
use thiserror::Error;

use crate::parse::Rule;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read config")]
    Config(#[from] io::Error),

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
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn custom_pest_error(message: String, span: Span) -> PestError<Rule> {
    PestError::new_from_span(PestErrorVariant::CustomError { message }, span)
}
