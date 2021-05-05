use std::str::FromStr;

use pest::error::Error as PestError;
use pest::error::ErrorVariant as PestErrorVariant;
use pest::iterators::Pair;

use crate::parse::Rule;

/// Define two structs that will hold config:
/// - Required fields will be stored as T; optional fields in an `Option<T>`
/// - All fields stored as `Option<T>`
///
/// Fields must implement: [Debug], [Clone], [FromStr].
macro_rules! define_config {
    // update optioned struct based on name and value parse pairs
    (
        @match_field_parse $self:expr, $name_pair:expr, $value_pair:expr, $( $field:ident )*
    ) => {
        let name_str = $name_pair.as_str();
        match name_str {
            $(
                stringify!($field) => {
                    let value = $value_pair.as_str()
                        .parse()
                        .map_err(|err| {
                            crate::Error::option_parse(name_str, &$value_pair, err)
                        })?;
                    $self.$field = Some(value);
                }
            )*
            _ => {
                return Err(PestError::new_from_span(
                    PestErrorVariant::CustomError {
                        message: format!("Invalid key {:?}", name_str),
                    },
                    $name_pair.as_span(),
                ).into());
            }
        }
    };
    (
        $( #[$struct_meta:meta] )*
        $struct_vis:vis struct $struct_name:ident, $opt_struct_name:ident;
        required {
            //port: u16,
            $(
                $( #[$req_field_meta:meta] )*
                $req_field_vis:vis $req_field:ident : $req_type:ty
            ),* $(,)?
        }
        optional {
            //uid: u32,
            $(
                $( #[$opt_field_meta:meta] )*
                $opt_field_vis:vis $opt_field:ident : $opt_type:ty
            ),* $(,)?
        }
    ) => {
        $( #[$struct_meta] )*
        $struct_vis struct $struct_name {
            pub name: String,
            $(
                $( #[$req_field_meta] )*
                $req_field_vis $req_field: $req_type,
            )*
            $(
                $( #[$opt_field_meta] )*
                $opt_field_vis $opt_field: Option<$opt_type>,
            )*
        }

        $( #[$struct_meta] )*
        #[derive(Default)]
        $struct_vis struct $opt_struct_name {
            $(
                $( #[$req_field_meta] )*
                $req_field_vis $req_field: Option<$req_type>,
            )*
            $(
                $( #[$opt_field_meta] )*
                $opt_field_vis $opt_field: Option<$opt_type>,
            )*
        }

        impl $struct_name {
            pub fn from_optioned(opt_struct: $opt_struct_name, service_name: &str, pair: &Pair<Rule>) -> crate::Result<Self> {
                Ok(Self {
                    name: service_name.to_string(),
                    $(
                        $req_field: opt_struct
                            .$req_field
                            .ok_or_else(|| {
                                let missing_option = stringify!($req_field);
                                crate::Error::missing_required_option(missing_option, service_name, pair)
                            })?,
                    )*
                    $(
                        $opt_field: opt_struct.$opt_field,
                    )*
                })
            }
        }

        impl $opt_struct_name {
            fn update_config(
                &mut self,
                name_pair: Pair<Rule>,
                value_pair: Pair<Rule>,
            ) -> Result<(), crate::Error> {
                assert_eq!(name_pair.as_rule(), Rule::name);
                assert_eq!(value_pair.as_rule(), Rule::value);

                define_config!(@match_field_parse self, name_pair, value_pair, $( $req_field )* $( $opt_field )*);

                Ok(())
            }

        }
    };
}

#[derive(Debug, Clone)]
pub struct ProgArgs(Vec<String>);

impl FromStr for ProgArgs {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let args: Vec<String> = s.split_whitespace().map(|s| s.to_string()).collect();
        Ok(Self(args))
    }
}

impl ServiceOption {
    pub fn update_from_body_pair(&mut self, body_pair: Pair<Rule>) -> crate::Result<()> {
        assert_eq!(body_pair.as_rule(), Rule::body);
        let property_pairs = body_pair.into_inner();
        for property_pair in property_pairs {
            assert_eq!(property_pair.as_rule(), Rule::property);
            let mut property_inner = property_pair.into_inner();
            let name_pair = property_inner.next().unwrap();
            let value_pair = property_inner.next().unwrap();
            self.update_config(name_pair, value_pair)?;
        }
        Ok(())
    }

    pub fn from_service_pair(service_pair: Pair<Rule>) -> crate::Result<Self> {
        let mut service_option = Self::default();
        service_option.update_from_body_pair(service_pair)?;
        Ok(service_option)
    }
}

define_config!(
    #[derive(Debug, Clone)]
    pub struct Service, ServiceOption;
    required {
        pub server: String,
        pub port: u16,
    }
    optional {
        pub uid: u32,
        pub server_args: ProgArgs,
    }
);
