use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use pest::error::Error as PestError;
use pest::error::ErrorVariant as PestErrorVariant;
use pest::iterators::Pair;

use crate::{
    config::{parse::Rule, InetType, ProgArgs, SocketType},
    Error,
};

/// Define two structs that will hold config:
/// - Required fields will be stored as T; optional fields in an `Option<T>`
/// - All fields stored as `Option<T>`
///
/// Fields must implement: [Debug], [Clone], [FromStr].
macro_rules! define_config {
    (
        @fill_defaults_set_fields $self:ident, $default_service:ident, $( $field:ident )*
    ) => {
        $(
            if $self.$field.is_none() {
                $self.$field = $default_service.$field.clone();
            }
        )*
    };

    // update optioned struct based on name and value parse pairs
    (
        @match_field_parse $self:expr, $struct_name:ident, $name_pair:expr,
        $value_pair:expr, $( $field:ident )*
    ) => {
        let name_str = $name_pair.as_str();
        match name_str {
            $(
                stringify!($field) => {
                    if $self.$field.is_some() {
                        return Err(crate::Error::duplicate_option(name_str, &$name_pair));
                    }
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
                        message: format!(
                            "Invalid key {:?}. Valid keys: {:?}",
                            name_str,
                            $struct_name::VALID_KEYS),
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
        optional_with_default {
            //socket_type: SocketType = SocketType::Tcp,
            $(
                $( #[$opt_def_field_meta:meta] )*
                $opt_def_field_vis:vis $opt_def_field:ident : $opt_def_type:ty = $opt_def_value:expr
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
                $( #[$opt_def_field_meta] )*
                $opt_def_field_vis $opt_def_field: $opt_def_type,
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
                $( #[$opt_def_field_meta] )*
                $opt_def_field_vis $opt_def_field: Option<$opt_def_type>,
            )*
            $(
                $( #[$opt_field_meta] )*
                $opt_field_vis $opt_field: Option<$opt_type>,
            )*
        }

        impl $struct_name {
            const VALID_KEYS: &'static [&'static str] = &[
                $( stringify!($req_field) , )*
                $( stringify!($opt_def_field) , )*
                $( stringify!($opt_field) , )*
            ];

            /// Convert from optioned struct. Required fields must be `Some(_)`.
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
                        $opt_def_field: opt_struct
                            .$opt_def_field
                            .unwrap_or($opt_def_value),
                    )*
                    $(
                        $opt_field: opt_struct.$opt_field,
                    )*
                })
            }
        }

        impl $opt_struct_name {
            /// Update field based on pairs
            fn update_config(
                &mut self,
                name_pair: Pair<Rule>,
                value_pair: Pair<Rule>,
            ) -> Result<(), crate::Error> {
                assert_eq!(name_pair.as_rule(), Rule::name);
                assert_eq!(value_pair.as_rule(), Rule::value);

                define_config!(@match_field_parse
                    self, $struct_name, name_pair, value_pair,
                    $( $req_field )* $( $opt_def_field )* $( $opt_field )*);

                Ok(())
            }

            /// For fields that are `None`, set them values defined by the default
            pub fn fill_with_defaults(&mut self, default_service: &$opt_struct_name) {
                define_config!(@fill_defaults_set_fields
                    self, default_service,
                    $( $req_field )* $( $opt_def_field )* $( $opt_field )* );
            }
        }
    };
}

impl Service {
    pub fn socket_addr(&self) -> crate::Result<SocketAddr> {
        let mismatch_err = |addr| {
            Err(Error::InetVersionAddressMismatch {
                addr,
                expected_type: self.inet_type,
                service_name: self.name.to_string(),
            })
        };
        let inet_addr: IpAddr = match self.inet_type {
            InetType::Ipv4 => {
                if let Some(addr) = self.listen_address {
                    if !addr.is_ipv4() {
                        return mismatch_err(addr);
                    }
                    addr
                } else {
                    Ipv4Addr::UNSPECIFIED.into()
                }
            }
            InetType::Ipv6 => {
                if let Some(addr) = self.listen_address {
                    if !addr.is_ipv6() {
                        return mismatch_err(addr);
                    }
                    addr
                } else {
                    Ipv6Addr::UNSPECIFIED.into()
                }
            }
        };

        Ok((inet_addr, self.port).into())
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
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Service, ServiceOption;
    required {
        /// Server binary
        pub server: String,

        /// TCP/UDP Port
        pub port: u16,
    }
    optional_with_default {
        /// Socket type (i.e., TCP vs. UDP)
        pub socket_type: SocketType = SocketType::Tcp,

        /// Inet (i.e., IPv4 vs. IPv6)
        pub inet_type: InetType = InetType::Ipv4,

        /// Program arguments
        pub server_args: ProgArgs = ProgArgs::default(),
    }
    optional {
        /// User ID to run the process
        pub uid: u32,

        /// IP address to listen on
        /// Defaults to all if not specified
        pub listen_address: IpAddr,
    }
);
