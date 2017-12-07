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
pub enum ChannelMode {
    I,
    S,
    W,
    O,
}

#[cfg(test)]
mod test {
    use rand;
    use super::{Message, ParseError};
    use super::commands::{Command, requests as Requests, responses as Responses};

    macro_rules! verify_parse {
        ($deserialized:expr, $raw:expr) => {
            assert_eq!($raw.parse::<Message>().unwrap(), $deserialized);
        }
    }

    macro_rules! verify_noparse{
        ($raw:expr) => {
            assert!($raw.parse::<Message>().is_err());
        };

        ($err_kind:expr, $raw:expr) => {
            assert_eq!($raw.parse::<Message>().err().unwrap(), $err_kind);
        }
    }

    #[test]
    fn test_parse() {
        verify_parse!(
            Message {
                prefix: Some("Laza".to_string()),
                command: Command::NICK(Requests::Nick { nickname: "lazau".to_string() }),
            },
            ":Laza NICK :lazau"
        );

        verify_parse!(
            Message {
                prefix: Some("Laza".to_string()),
                command: Command::NICK(Requests::Nick { nickname: "lazau".to_string() }),
            },
            ":Laza NICK   :lazau"
        );

        verify_parse!(
            Message {
                prefix: None,
                command: Command::NICK(Requests::Nick { nickname: "lazau".to_string() }),
            },
            "NICK      lazau"
        );

        verify_parse!(
            Message {
                prefix: None,
                command: Command::USER(Requests::User {
                    username: "d".to_string(),
                    mode: "0".to_string(),
                    unused: "d".to_string(),
                    realname: "g".to_string(),
                }),
            },
            "USER d 0 d g"
        );

        verify_parse!(
            Message {
                prefix: None,
                command: Command::ERR_NEEDMOREPARAMS(Responses::NeedMoreParams::default()),
            },
            "461"
        )
    }

    #[test]
    fn test_parse_failure() {
        verify_noparse!(ParseError::NoCommand, ":Hello");
        verify_noparse!(ParseError::NoCommand, ":");
        verify_noparse!(ParseError::NoCommand, "");
        verify_noparse!(ParseError::UnrecognizedCommand, "whatacommand");
        verify_noparse!(ParseError::UnrecognizedCommand, ":a whatacommand sd dd :ee");
    }

    #[test]
    fn fuzz_parser() {
        let (max_input_len, cases) = (1024, 1000);
        for _ in 0..cases {
            let len = rand::random::<u32>() % max_input_len;
            let mut input = String::with_capacity(len as usize);
            for _ in 0..len {
                input.push(rand::random::<char>());
            }
            println!("Testing {} : {:?}", input, input.parse::<Message>());
        }
    }

    macro_rules! verify_serialize {
        ($output:expr, $input:expr) => {
            assert_eq!($output, format!("{}", $input))
        }
    }

    #[test]
    fn test_serialize() {
        verify_serialize!(
            ":test 001 lazau :Hello world!",
            Message {
                prefix: Some("test".to_string()),
                command: Command::RPL_WELCOME(Responses::Welcome {
                    nick: "lazau".to_string(),
                    message: "Hello world!".to_string(),
                }),
            }
        );

        verify_serialize!(
            ":test.server.com 461 JOIN :Not enough parameters",
            Message {
                prefix: Some("test.server.com".to_string()),
                command: Command::ERR_NEEDMOREPARAMS(
                    Responses::NeedMoreParams { command: "JOIN".to_string() },
                ),
            }
        );

    }
}
