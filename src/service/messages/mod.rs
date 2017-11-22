mod responses;
mod requests;

pub use self::responses::Response;
pub use self::requests::Request;

use std;
use std::fmt;
use std::str;

#[derive(Debug)]
pub enum ParseErrorKind {
    NoCommand,
    UnrecognizedCommand,
    TooFewParams,
    TooManyParams,
    Other,
}

#[derive(Debug)]
pub struct ParseError {
    desc: &'static str,
    kind: ParseErrorKind,
}

#[derive(Debug)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: Command,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Req(requests::Request),
    Resp(responses::Response),
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, desc: &'static str) -> ParseError {
        ParseError {
            desc: desc,
            kind: kind,
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        self.desc
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IRC command parse error: {}", &self.desc)
    }
}

impl str::FromStr for Message {
    type Err = ParseError;
    // RFC 1459 2
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 1 || s.len() > 510 {
            return Err(ParseError::new(ParseErrorKind::Other, "bad command length"));
        }

        let mut remainder: &str = &s.trim_right();
        debug!("Processing {:?}", remainder);

        let mut prefix: Option<String> = None;
        if remainder.starts_with(':') {
            match remainder.find(' ') {
                Some(idx) => {
                    prefix = Some(remainder[0..idx].to_string());
                    remainder = &remainder[idx + 1..];
                }
                None => {
                    return Err(ParseError::new(
                        ParseErrorKind::NoCommand,
                        "only prefix given",
                    ));
                }
            }
        }
        if remainder.len() < 1 {
            return Err(ParseError::new(
                ParseErrorKind::NoCommand,
                "no command specified",
            ));
        }

        let command: Command = Command::Req(remainder.parse::<Request>()?);
        debug!(
            "Parsed {} to prefix: [{:?}]; command: [{:?}].",
            s,
            prefix,
            command,
        );

        Ok(Message {
            prefix: prefix,
            command: command,
        })
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref prefix) = self.prefix {
            assert!(prefix.find(' ').is_none());
            write!(f, ":{} ", prefix)?;
        }
        match self.command {
            // TODO(lazau): Maybe don't panic here?
            Command::Req(_) => panic!("Attempting to display client request."),
            Command::Resp(ref r) => write!(f, "{}", r)?,
        };
        /*if self.params.len() > 0 {
            write!(f, " ")?;
            for p in self.params.iter().take(self.params.len() - 1) {
                // TODO(lazau): Maybe just split it into more params rather than panicking?
                assert!(p.find(' ').is_none());
                write!(f, "{} ", p)?;
            }
            write!(f, ":{}", self.params[self.params.len() - 1])?;
        }*/
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Command, Message, Request};

    macro_rules! test_message_fail {
        ($name:ident, $s:expr) => {
            #[test]
            fn $name() {
                assert!((&format!("{}\r\n", $s)).parse::<Message>().is_err());
            }
        }
    }
    macro_rules! test_message_pass {
        ($name:ident, $input:expr, Message {
            prefix: $prefix:expr,
            command: $command:expr,
            params: [$($params:expr),*],
        }) => {
            #[test]
            fn $name() {
                let s = (&format!("{}\r\n",$input)).parse::<Message>().unwrap();
                let pf = $prefix.to_string();
                if pf.len() == 0 {
                    assert!(s.prefix.is_none());
                } else {
                    assert_eq!(s.prefix.unwrap(), $prefix.to_string());
                }
                assert_eq!(s.command, $command);
                let params:Vec<&str> = vec![$($params),*];
                let expect :Vec<String> = params.iter().map(|s| s.to_string()).collect();
                assert_eq!(expect.len(), s.params.len());
                expect.iter().zip(s.params.iter()).for_each(|p| assert_eq!(p.0, p.1));
            }
        }
    }

    test_message_fail!(empty, "");
    test_message_fail!(just_prefix, ":lazau");

    test_message_pass!(
        hello_world,
        "NICK world",
        Message {
            prefix: "",
            command: Command::Req(Request::NICK),
            params: ["world"],
        }
    );
    test_message_pass!(
        empty_param,
        "PASS",
        Message {
            prefix: "",
            command: Command::Req(Request::PASS),
            params: [],
        }
    );
    test_message_pass!(
        empty_param_trailer,
        "QUIT :",
        Message {
            prefix: "",
            command: Command::Req(Request::QUIT),
            params: [],
        }
    );
    test_message_pass!(
        full,
        ":lazau CONNECT server server2 :server 3 5 6",
        Message {
            prefix: ":lazau",
            command: Command::Req(Request::CONNECT),
            params: ["server", "server2", "server 3 5 6"],
        }
    );
}
