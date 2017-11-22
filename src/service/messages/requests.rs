use std::fmt;
use std::str;

// RFC 1459 4, 5.
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Clone)]
pub enum Request {
    // 4.1 Connection Registration.
    NICK,
    PASS,
    USER,
    SERVER,
    OPER,
    QUIT,
    SQUIT,

    // 4.2 Channel Operations.
    JOIN,
    PART,
    MODE,
    TOPIC,
    NAMES,
    LIST,
    INVITE,
    KICK,

    // 4.3 Server queries and commands.
    VERSION,
    STATS,
    LINKS,
    TIME,
    CONNECT,
    TRACE,
    ADMIN,
    INFO,

    // 4.4 Sending messages.
    PRIVMSG,
    NOTICE,

    // 4.5 User based queries.
    WHO,
    WHOIS,
    WHOWAS,

    // 4.6 Misc.
    KILL,
    PING,
    PONG,
    ERROR,

    // 5 Optionals.
    AWAY,
    REHASH,
    RESTART,
    SUMMON,
    USERS,
    WALLOPS,
    USERHOST,
    ISON,
}

#[allow(non_snake_case)]
impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Request::NICK => "NICK",
                &Request::PASS => "PASS",
                &Request::USER => "USER",
                &Request::SERVER => "SERVER",
                &Request::OPER => "OPER",
                &Request::QUIT => "QUIT",
                &Request::SQUIT => "SQUIT",
                &Request::JOIN => "JOIN",
                &Request::PART => "PART",
                &Request::MODE => "MODE",
                &Request::TOPIC => "TOPIC",
                &Request::NAMES => "NAMES",
                &Request::LIST => "LIST",
                &Request::INVITE => "INVITE",
                &Request::KICK => "KICK",
                &Request::VERSION => "VERSION",
                &Request::STATS => "STATS",
                &Request::LINKS => "LINKS",
                &Request::TIME => "TIME",
                &Request::CONNECT => "CONNECT",
                &Request::TRACE => "TRACE",
                &Request::ADMIN => "ADMIN",
                &Request::INFO => "INFO",
                &Request::PRIVMSG => "PRIVMSG",
                &Request::NOTICE => "NOTICE",
                &Request::WHO => "WHO",
                &Request::WHOIS => "WHOIS",
                &Request::WHOWAS => "WHOWAS",
                &Request::KILL => "KILL",
                &Request::PING => "PING",
                &Request::PONG => "PONG",
                &Request::ERROR => "ERROR",
                &Request::AWAY => "AWAY",
                &Request::REHASH => "REHASH",
                &Request::RESTART => "RESTART",
                &Request::SUMMON => "SUMMON",
                &Request::USERS => "USERS",
                &Request::WALLOPS => "WALLOPS",
                &Request::USERHOST => "USERHOST",
                &Request::ISON => "ISON",
            }
        )
    }
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
            "NICK" => Ok(Request::NICK),
            "PASS" => Ok(Request::PASS),
            "USER" => Ok(Request::USER),
            "SERVER" => Ok(Request::SERVER),
            "OPER" => Ok(Request::OPER),
            "QUIT" => Ok(Request::QUIT),
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
