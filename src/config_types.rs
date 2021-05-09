use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProgArgs(pub Vec<String>);

impl FromStr for ProgArgs {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let args = shlex::split(s).ok_or("Invalid shell escaped string")?;
        Ok(Self(args))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    /// "stream"
    Tcp,

    /// "dgram"
    Udp,
}

impl FromStr for SocketType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcp" | "stream" => Ok(Self::Tcp),
            "udp" | "dgram" => Ok(Self::Udp),
            _ => Err("Invalid input: must be tcp|udp (or alises stream|dgram)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InetType {
    /// IPv4
    Ipv4,

    /// IPv6
    Ipv6,
}

impl FromStr for InetType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ipv4" => Ok(Self::Ipv4),
            "ipv6" => Ok(Self::Ipv6),
            _ => Err("Invalid input: must be ipv4|ipv6"),
        }
    }
}

impl Display for InetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4 => write!(f, "IPv4"),
            Self::Ipv6 => write!(f, "IPv6"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn socket_type() {
        assert_eq!("tcp".parse::<SocketType>(), Ok(SocketType::Tcp));
        assert_eq!("UDP".parse::<SocketType>(), Ok(SocketType::Udp));
        assert!("blah".parse::<SocketType>().is_err());
    }

    #[test]
    fn prog_args() {
        assert_eq!("".parse::<ProgArgs>(), Ok(ProgArgs(vec![])));
        assert_eq!(
            "simple".parse::<ProgArgs>(),
            Ok(ProgArgs(vec!["simple".to_string()]))
        );
        assert_eq!(
            "\t\t  arg1 arg2\targ3  ".parse::<ProgArgs>(),
            Ok(ProgArgs(vec![
                "arg1".to_string(),
                "arg2".to_string(),
                "arg3".to_string()
            ]))
        );
        assert_eq!(
            r#"'arg1 with spaces' arg2\ with\ \ spaces "with quotes""#.parse::<ProgArgs>(),
            Ok(ProgArgs(vec![
                "arg1 with spaces".to_string(),
                "arg2 with  spaces".to_string(),
                "with quotes".to_string(),
            ]))
        );
    }

    #[test]
    fn inet_type() {
        assert_eq!("ipv4".parse::<InetType>(), Ok(InetType::Ipv4));
        assert_eq!("IPv4".parse::<InetType>(), Ok(InetType::Ipv4));
        assert_eq!("ipv6".parse::<InetType>(), Ok(InetType::Ipv6));
        assert_eq!("IPv6".parse::<InetType>(), Ok(InetType::Ipv6));
        assert!("IPv99".parse::<InetType>().is_err());
    }
}
