use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgArgs(Vec<String>);

impl FromStr for ProgArgs {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // todo(tmfink): handle shell quoting
        let args: Vec<String> = s.split_whitespace().map(|s| s.to_string()).collect();
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
