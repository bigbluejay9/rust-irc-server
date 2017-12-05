pub mod commands;

pub use self::commands::Command;

use std::{self, fmt, str};
use std::fmt::{Formatter, Error as FmtError};

fn next_token<'a>(s: &'a str) -> (&'a str, &'a str) {
    match s.find(' ') {
        Some(idx) => {
            let (a, b) = s.split_at(idx);
            (a, &b[1..])
        }
        None => (s, ""),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    NoCommand,
    UnrecognizedCommand,
    NeedMoreParams { command: String },
    NotARequest,
    NotAResponse,
    Other { desc: String },
}

impl<T: std::error::Error> std::convert::From<T> for ParseError {
    fn from(p: T) -> Self {
        ParseError::Other { desc: p.description().to_string() }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error: {:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: commands::Command,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match &self.prefix {
            &Some(ref p) => write!(f, ":{} {}", p, self.command),
            &None => write!(f, "{}", self.command),
        }
    }
}

impl str::FromStr for Message {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        trace!("Parsing {} to Message.", s);

        let mut remainder: &str = &s;
        let mut prefix = None;
        if s.starts_with(":") {
            let (a, b) = next_token(s);
            remainder = b;
            prefix = Some(a[1..].to_string());
        }

        if remainder.len() == 0 {
            return Err(ParseError::NoCommand);
        }
        let command = remainder.parse::<commands::Command>()?;

        Ok(Message {
            prefix: prefix,
            command: command,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum UserMode {
    O,
    P,
    S,
    I,
    T,
    N,
    M,
    L,
    B,
    V,
    K,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum ChannelMode {
    I,
    S,
    W,
    O,
}
