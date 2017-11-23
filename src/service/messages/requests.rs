use std::fmt;
use std::str;
use std::collections::HashSet;

use super::{ParseError, ParseErrorKind};
use super::serialize_params;

pub type UserModes = HashSet<UserMode>;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UserMode {
    o,
    p,
    s,
    i,
    t,
    n,
    m,
    l,
    b,
    v,
    k,
}

pub type ChannelModes = HashSet<ChannelMode>;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ChannelMode {
    i,
    s,
    w,
    o,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModeModifier {
    Add,
    Sub,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RequestMode {
    Channel {
        channel: String,
        op: ModeModifier,
        modes: ChannelMode,
        limit: Option<String>,
        user: Option<String>,
        ban_mask: Option<String>,
    },
    User {
        nickname: String,
        op: ModeModifier,
        modes: UserModes,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StatsQuery {
    c,
    h,
    i,
    k,
    l,
    m,
    o,
    y,
    u,
}

// RFC 1459 4, 5. RFC 2812.
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Request {
    // 4.1 Connection Registration.
    NICK { nickname: String },
    PASS { password: String },
    USER {
        username: String,
        mode: UserMode,
        unused: String,
        realname: String,
    },
    SERVER {
        servername: String,
        hopcount: u64,
        token: u64,
        info: String,
    },
    OPER { name: String, password: String },
    QUIT { message: Option<String> },
    SQUIT { server: String, comment: String },

    // 4.2 Channel Operations.
    JOIN {
        part_all: bool,
        channels: Vec<(String, Option<String>)>,
    },
    PART {
        channels: Vec<String>,
        message: Option<String>,
    },
    // TODO(lazau): Verify.
    MODE { request_mode: RequestMode },
    TOPIC {
        channel: String,
        topic: Option<String>,
    },
    NAMES { channels: Vec<String> },
    LIST {
        channels: Option<Vec<String>>,
        elist: Option<Vec<String>>,
    },
    INVITE { nickname: String, channel: String },
    KICK {
        channel: Vec<String>,
        user: Vec<String>,
        comment: Option<String>,
    },

    // 4.3 Server queries and commands.
    VERSION { target: Option<String> },
    STATS { query: StatsQuery, target: String },
    // TODO(lazau): Server mask should be a type.
    LINKS {
        remote_server: String,
        server_mask: String,
    },
    TIME { target: Option<String> },
    CONNECT {
        target: String,
        port: Option<u32>,
        remote: Option<String>,
    },
    TRACE { target: Option<String> },
    ADMIN { target: Option<String> },
    INFO { target: Option<String> },

    // 4.4 Sending messages.
    PRIVMSG { targets: Vec<String>, text: String },
    NOTICE { targets: Vec<String>, text: String },

    // 4.5 User based queries.
    WHO { mask: String, operators: bool },
    WHOIS {
        target: Option<String>,
        masks: Vec<String>,
    },
    WHOWAS {
        nicknames: Vec<String>,
        max: Option<i64>,
        target: Option<String>,
    },

    // 4.6 Misc.
    KILL { nickname: String, comment: String },
    PING {
        server1: String,
        server2: Option<String>,
    },
    PONG {
        daemon1: String,
        daemon2: Option<String>,
    },
    ERROR { message: String },

    // 5 Optionals.
    AWAY { message: Option<String> },
    REHASH,
    RESTART,
    SUMMON {
        user: String,
        target: Option<String>,
        channel: Option<String>,
    },
    USERS { target: Option<String> },
    WALLOPS { text: String },
    USERHOST { nicknames: Vec<String> },
    ISON { nicknames: Vec<String> },
}

#[allow(non_snake_case)]
impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Request::NICK { nickname: ref nick } => {
                write!(f, "NICK {}", serialize_params(vec![nick])?)
            }
            &Request::PASS { password: ref pass } => {
                write!(f, "PASS {}", serialize_params(vec![pass])?)
            }
            &Request::USER {
                username: ref n,
                mode: ref h,
                unused: ref s,
                realname: ref r,
            } => write!(f, "USER {}", serialize_params(vec![n, h, s, r])?),
            &Request::SERVER {
                servername: ref s,
                hopcount: ref h,
                token: ref t,
                info: ref i,
            } => {
                write!(
                    f,
                    "SERVER {}",
                    serialize_params(vec![s, &h.to_string(), &t.to_string(), i])?
                )
            }
            &Request::OPER {
                name: ref n,
                password: ref p,
            } => write!(f, "OPER {}", serialize_params(vec![n, p])?),
            &Request::QUIT { message: ref m } => {
                match m {
                    &Some(ref s) => write!(f, "QUIT {}", serialize_params(vec![s])?),
                    &None => write!(f, "QUIT"),
                }
            }
            &Request::SQUIT => write!(f, "SQUIT"),
            &Request::JOIN => write!(f, "JOIN"),
            &Request::PART => write!(f, "PART"),
            &Request::MODE => write!(f, "MODE"),
            &Request::TOPIC => write!(f, "TOPIC"),
            &Request::NAMES => write!(f, "NAMES"),
            &Request::LIST => write!(f, "LIST"),
            &Request::INVITE => write!(f, "INVITE"),
            &Request::KICK => write!(f, "KICK"),
            &Request::VERSION => write!(f, "VERSION"),
            &Request::STATS => write!(f, "STATS"),
            &Request::LINKS => write!(f, "LINKS"),
            &Request::TIME => write!(f, "TIME"),
            &Request::CONNECT => write!(f, "CONNECT"),
            &Request::TRACE => write!(f, "TRACE"),
            &Request::ADMIN => write!(f, "ADMIN"),
            &Request::INFO => write!(f, "INFO"),
            &Request::PRIVMSG => write!(f, "PRIVMSG"),
            &Request::NOTICE => write!(f, "NOTICE"),
            &Request::WHO => write!(f, "WHO"),
            &Request::WHOIS => write!(f, "WHOIS"),
            &Request::WHOWAS => write!(f, "WHOWAS"),
            &Request::KILL => write!(f, "KILL"),
            &Request::PING => write!(f, "PING"),
            &Request::PONG => write!(f, "PONG"),
            &Request::ERROR => write!(f, "ERROR"),
            &Request::AWAY => write!(f, "AWAY"),
            &Request::REHASH => write!(f, "REHASH"),
            &Request::RESTART => write!(f, "RESTART"),
            &Request::SUMMON => write!(f, "SUMMON"),
            &Request::USERS => write!(f, "USERS"),
            &Request::WALLOPS => write!(f, "WALLOPS"),
            &Request::USERHOST => write!(f, "USERHOST"),
            &Request::ISON => write!(f, "ISON"),
        }
    }
}

fn verify_at_least_params(
    p: &Vec<String>,
    required: usize,
    error: &'static str,
) -> Result<(), ParseError> {
    if p.len() < required {
        return Err(ParseError::new(ParseErrorKind::NeedMoreParams, error));
    }
    Ok(())
}

impl str::FromStr for Request {
    type Err = super::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug!("Parsing {} as a client request.", s);

        let mut remainder: &str = &s;

        let next_space;
        let command_str: &str;
        match remainder.find(' ') {
            Some(idx) => {
                command_str = &remainder[0..idx];
                remainder = &remainder[idx + 1..];
            }
            None => {
                command_str = remainder;
                remainder = "";
            }
        };

        let mut params: Vec<String> = Vec::new();
        while remainder.len() > 0 {
            if remainder.starts_with(':') {
                if remainder.len() == 1 {
                    warn!("Empty trailing command parameter. Ignoring.")
                } else {
                    params.push(remainder[1..].to_string());
                }
                break;
            }
            match remainder.find(' ') {
                Some(idx) => {
                    if idx == 0 {
                        warn!("Empty whitespace in command paramter detected! Ignoring.");
                    } else {
                        params.push(remainder[0..idx].to_string());
                    }
                    remainder = &remainder[idx + 1..];
                }
                None => {
                    params.push(remainder.to_string());
                    break;
                }
            }
        }

        // TODO(lazau): Parse params.
        match command_str.to_uppercase().as_ref() {
            "NICK" => {
                verify_at_least_params(&params, 1, "NICK")?;
                Ok(Request::NICK { nickname: params[0] })
            }
            "PASS" => {
                verify_at_least_params(&params, 1, "PASS")?;
                Ok(Request::PASS { password: params[0] })
            }
            "USER" => {
                verify_at_least_params(&params, 4, "USER")?;
                Ok(Request::USER {
                    username: params[0],
                    hostname: params[1],
                    servername: params[2],
                    realname: params[3],
                })
            }
            "SERVER" => {
                verify_at_least_params(&params, 3, "USER")?;
                Ok(Request::SERVER {
                    servername: params[0],
                    hopcount: match params[1].parse::<u64>() {
                        Ok(h) => h,
                        Err(e) => {
                            return Err(ParseError::new(
                                ParseErrorKind::ParseIntError,
                                "hopcount not an int",
                            ))
                        }
                    },
                    info: params[2],
                })
            }
            "OPER" => {
                verify_at_least_params(&params, 2, "OPER")?;
                Ok(Request::OPER {
                    name: params[0],
                    password: params[1],
                })
            }
            "QUIT" => {
                if params.len() == 0 {
                    return Ok(Request::QUIT { message: None });
                }
                Ok(Request::QUIT { message: Some(params[0]) })
            }
            "SQUIT" => Ok(Request::SQUIT),
            "JOIN" => Ok(Request::JOIN),
            "PART" => Ok(Request::PART),
            "MODE" => Ok(Request::MODE),
            "TOPIC" => Ok(Request::TOPIC),
            "NAMES" => Ok(Request::NAMES),
            "LIST" => Ok(Request::LIST),
            "INVITE" => Ok(Request::INVITE),
            "KICK" => Ok(Request::KICK),
            "VERSION" => Ok(Request::VERSION),
            "STATS" => Ok(Request::STATS),
            "LINKS" => Ok(Request::LINKS),
            "TIME" => Ok(Request::TIME),
            "CONNECT" => Ok(Request::CONNECT),
            "TRACE" => Ok(Request::TRACE),
            "ADMIN" => Ok(Request::ADMIN),
            "INFO" => Ok(Request::INFO),
            "PRIVMSG" => Ok(Request::PRIVMSG),
            "NOTICE" => Ok(Request::NOTICE),
            "WHO" => Ok(Request::WHO),
            "WHOIS" => Ok(Request::WHOIS),
            "WHOWAS" => Ok(Request::WHOWAS),
            "KILL" => Ok(Request::KILL),
            "PING" => Ok(Request::PING),
            "PONG" => Ok(Request::PONG),
            "ERROR" => Ok(Request::ERROR),
            "AWAY" => Ok(Request::AWAY),
            "REHASH" => Ok(Request::REHASH),
            "RESTART" => Ok(Request::RESTART),
            "SUMMON" => Ok(Request::SUMMON),
            "USERS" => Ok(Request::USERS),
            "WALLOPS" => Ok(Request::WALLOPS),
            "USERHOST" => Ok(Request::USERHOST),
            "ISON" => Ok(Request::ISON),
            _ => Err(super::ParseError::new(
                super::ParseErrorKind::UnrecognizedCommand,
                "unrecognized command",
            )),
        }
    }
}
