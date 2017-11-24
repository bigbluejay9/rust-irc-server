use std::fmt;
use std::str;
use std::collections::HashSet;

use super::{ParseError, ParseErrorKind};
use super::serialize_params;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserModes {
    modes: HashSet<UserMode>,
}

impl fmt::Display for UserModes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

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

impl fmt::Display for UserMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
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
pub enum RequestedMode {
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

impl fmt::Display for RequestedMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
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

impl fmt::Display for StatsQuery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum JoinChannels {
    Channels(Vec<String>),
    KeyedChannels(Vec<(String, String)>),
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
        channels: JoinChannels,
    },
    PART {
        channels: Vec<String>,
        message: Option<String>,
    },
    // TODO(lazau): Verify.
    MODE { mode: RequestedMode },
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
    STATS {
        query: Option<StatsQuery>,
        target: Option<String>,
    },
    // TODO(lazau): Server mask should be a type.
    LINKS {
        remote_server: Option<String>,
        server_mask: Option<String>,
    },
    TIME { target: Option<String> },
    CONNECT {
        target: String,
        port: u32,
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
        let out = match self {
            &Request::NICK { nickname: nick } => format!("NICK {}", serialize_params(&vec![nick])?),

            &Request::PASS { password: pass } => format!("PASS {}", serialize_params(&vec![pass])?),

            &Request::USER {
                username: n,
                mode: h,
                unused: s,
                realname: r,
            } => format!("USER {}", serialize_params(&vec![n, h.to_string(), s, r])?),

            &Request::SERVER {
                servername: s,
                hopcount: h,
                token: t,
                info: i,
            } => {
                format!(
                    "SERVER {}",
                    serialize_params(&vec![s, h.to_string(), t.to_string(), i])?
                )
            }

            &Request::OPER {
                name: n,
                password: p,
            } => format!("OPER {}", serialize_params(&vec![n, p])?),

            &Request::QUIT { message: m } => {
                match m {
                    Some(s) => format!("QUIT {}", serialize_params(&vec![s])?),
                    None => format!("QUIT"),
                }
            }

            &Request::SQUIT {
                server: s,
                comment: c,
            } => format!("SQUIT {}", serialize_params(&vec![s, c])?),

            &Request::JOIN {
                part_all: p,
                channels: jc,
            } => {
                if p {
                    format!("JOIN 0")
                } else {
                    let params = Vec::new();
                    match jc {
                        JoinChannels::Channels(c) => {
                            params.push(c.join(","));
                        }
                        JoinChannels::KeyedChannels(kc) => {
                            let (channels, keys): (Vec<String>,
                                                   Vec<String>) = kc.iter().cloned().unzip();
                            params.push(channels.join(","));
                            params.push(keys.join(","));
                        }
                    };
                    format!("JOIN {}", serialize_params(&params)?)
                }
            }

            &Request::PART {
                channels: c,
                message: m,
            } => {
                let params = Vec::new();
                params.push(c.join(","));
                match m {
                    Some(m) => params.push(m),
                    None => {}
                }
                format!("PART {}", serialize_params(&params)?)
            }

            &Request::MODE { mode: m } => {
                format!("MODE {}", serialize_params(&vec![m.to_string()])?)
            }

            &Request::TOPIC {
                channel: c,
                topic: t,
            } => {
                let params = Vec::new();
                params.push(c);
                match t {
                    Some(t) => params.push(t),
                    None => {}
                }
                format!("TOPIC {}", serialize_params(&params)?)
            }

            &Request::NAMES { channels: c } => format!("NAMES {}", serialize_params(&c)?),

            &Request::LIST {
                channels: c,
                elist: e,
            } => {
                let params = Vec::new();
                match c {
                    Some(c) => {
                        params.push(c.join(","));
                        match e {
                            Some(e) => params.push(e.join(",")),
                            None => {}
                        }
                    }
                    None => {}
                }
                format!("LIST {}", serialize_params(&params)?)
            }

            &Request::INVITE {
                nickname: n,
                channel: c,
            } => format!("INVITE {}", serialize_params(&vec![n, c])?),

            &Request::KICK {
                channel: c,
                user: u,
                comment: co,
            } => {
                let params = Vec::new();
                params.push(c.join(","));
                params.push(u.join(","));
                match co {
                    Some(co) => params.push(co),
                    None => {}
                }
                format!("KICK {}", serialize_params(&params)?)
            }

            &Request::VERSION { target: t } => {
                let params = Vec::new();
                match t {
                    Some(t) => params.push(t),
                    None => {}
                }
                format!("VERSION {}", serialize_params(&params)?)
            }

            &Request::STATS {
                query: sq,
                target: t,
            } => {
                let params = Vec::new();
                match sq {
                    Some(sq) => {
                        params.push(sq.to_string());
                        match t {
                            Some(t) => params.push(t),
                            None => {}
                        }
                    }
                    None => {}
                }
                format!("STATS {}", serialize_params(&params)?)
            }

            &Request::LINKS {
                remote_server: r,
                server_mask: s,
            } => {
                let params = Vec::new();
                match s {
                    Some(s) => {
                        params.push(s);
                        match r {
                            Some(r) => params.push(r),
                            None => {}
                        }
                    }
                    None => {}
                }
                format!("LINKS {}", serialize_params(&params)?)
            }

            &Request::TIME { target: t } => {
                let params = Vec::new();
                match t {
                    Some(t) => params.push(t),
                    None => {}
                }
                format!("TIME {}", serialize_params(&params)?)
            }

            &Request::CONNECT {
                target: t,
                port: p,
                remote: r,
            } => {
                let params = vec![t, p.to_string()];
                match r {
                    Some(r) => params.push(r),
                    None => {}
                }
                format!("CONNECT {}", serialize_params(&params)?)
            }

            &Request::TRACE { target: t } => {
                let params = Vec::new();
                match t {
                    Some(t) => params.push(t),
                    None => {}
                }
                format!("TRACE {}", serialize_params(&params)?)
            }

            &Request::ADMIN { target: t } => {
                let params = Vec::new();
                match t {
                    Some(t) => params.push(t),
                    None => {}
                }
                format!("ADMIN {}", serialize_params(&params)?)
            }

            &Request::INFO { target: t } => {
                let params = Vec::new();
                match t {
                    Some(t) => params.push(t),
                    None => {}
                }
                format!("INFO {}", serialize_params(&params)?)
            }

            &Request::PRIVMSG {
                targets: t,
                text: x,
            } => {
                let params = vec![t.join(","), x];
                format!("PRIVMSG {}", serialize_params(&params)?)
            }

            &Request::NOTICE {
                targets: t,
                text: x,
            } => {
                let params = vec![t.join(","), x];
                format!("NOTICE {}", serialize_params(&params)?)
            }

            &Request::WHO {
                mask: m,
                operators: o,
            } => {
                let params = vec![m];
                if o {
                    params.push("o".to_string());
                }
                format!("WHO {}", serialize_params(&params)?)
            }

            &Request::WHOIS {
                target: t,
                masks: m,
            } => {
                let params = Vec::new();
                match t {
                    Some(t) => params.push(t),
                    None => {}
                }
                params.push(m.join(","));
                format!("WHOIS {}", serialize_params(&params)?)
            }

            &Request::WHOWAS {
                nicknames: n,
                max: m,
                target: t,
            } => {
                let params = vec![n.join(",")];
                if m.is_some() {
                    params.push(m.unwrap().to_string());
                    if t.is_some() {
                        params.push(t.unwrap().to_string());
                    }
                }
                format!("WHOWAS {}", serialize_params(&params)?)
            }

            &Request::KILL {
                nickname: n,
                comment: c,
            } => {
                let params = vec![n, c];
                format!("KILL {}", serialize_params(&params)?)
            }

            &Request::PING {
                server1: s1,
                server2: s2,
            } => {
                let params = vec![s1];
                if s2.is_some() {
                    params.push(s2.unwrap());
                }
                format!("PING {}", serialize_params(&params)?)
            }

            &Request::PONG {
                daemon1: d1,
                daemon2: d2,
            } => {
                let params = vec![d1];
                if d2.is_some() {
                    params.push(d2.unwrap());
                }
                format!("PONG {}", serialize_params(&params)?)
            }

            &Request::ERROR { message: m } => {
                let params = vec![m];
                format!("ERROR {}", serialize_params(&params)?)
            }

            &Request::AWAY { message: m } => {
                let params = Vec::new();
                if m.is_some() {
                    params.push(m.unwrap());
                }
                format!("AWAY {}", serialize_params(&params)?)
            }

            &Request::REHASH => format!("REHASH"),

            &Request::RESTART => format!("RESTART"),

            &Request::SUMMON {
                user: u,
                target: t,
                channel: c,
            } => {
                let params = vec![u];
                if t.is_some() {
                    params.push(t.unwrap());
                    if c.is_some() {
                        params.push(c.unwrap());
                    }
                }
                format!("SUMMON {}", serialize_params(&params)?)
            }

            &Request::USERS { target: t } => {
                let params = Vec::new();
                if t.is_some() {
                    params.push(t.unwrap());
                }
                format!("USERS {}", serialize_params(&params)?)
            }

            &Request::WALLOPS { text: t } => {
                let params = vec![t];
                format!("WALLOPS {}", serialize_params(&params)?)
            }

            &Request::USERHOST { nicknames: n } => {
                let params = vec![n.join(",")];
                format!("USERHOST {}", serialize_params(&params)?)
            }

            &Request::ISON { nicknames: n } => {
                let params = vec![n.join(",")];
                format!("ISON {}", serialize_params(&params)?)
            }
        };

        // Allows us to generate commands with trailing spaces (in the case of empty optional
        // params).
        write!(f, "{}", out.trim_right())
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
