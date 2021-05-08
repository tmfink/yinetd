use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgArgs(Vec<String>);

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
}
