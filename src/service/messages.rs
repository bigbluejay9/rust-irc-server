use std;
use std::fmt;
use std::str;

#[derive(Debug)]
pub struct ParseError {
    desc: &'static str,
}

#[derive(Debug)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: Command,
    pub params: Vec<String>,
}

// RFC 1459 4, 5.
#[allow(non_snake_case)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Command {
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

impl ParseError {
    pub fn new(desc: &'static str) -> ParseError {
        ParseError { desc: desc }
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
            return Err(ParseError::new("bad command length"));
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
                    return Err(ParseError::new("only command prefix given"));
                }
            }
        }

        if remainder.len() < 1 {
            return Err(ParseError::new("no command specified"));
        }
        let command: Command;
        match remainder.find(' ') {
            Some(idx) => {
                command = remainder[0..idx].parse::<Command>()?;
                remainder = &remainder[idx + 1..];
            }
            None => {
                command = remainder.parse::<Command>()?;
                remainder = "";
            }
        }

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

        debug!(
            "Parsed {} to prefix: [{:?}]; command: [{}]; params: [{:?}].",
            s,
            prefix,
            command,
            params
        );

        Ok(Message {
            prefix: prefix,
            command: command,
            params: params,
        })
    }
}

#[allow(non_snake_case)]
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Command::NICK => "NICK",
                &Command::PASS => "PASS",
                &Command::USER => "USER",
                &Command::SERVER => "SERVER",
                &Command::OPER => "OPER",
                &Command::QUIT => "QUIT",
                &Command::SQUIT => "SQUIT",
                &Command::JOIN => "JOIN",
                &Command::PART => "PART",
                &Command::MODE => "MODE",
                &Command::TOPIC => "TOPIC",
                &Command::NAMES => "NAMES",
                &Command::LIST => "LIST",
                &Command::INVITE => "INVITE",
                &Command::KICK => "KICK",
                &Command::VERSION => "VERSION",
                &Command::STATS => "STATS",
                &Command::LINKS => "LINKS",
                &Command::TIME => "TIME",
                &Command::CONNECT => "CONNECT",
                &Command::TRACE => "TRACE",
                &Command::ADMIN => "ADMIN",
                &Command::INFO => "INFO",
                &Command::PRIVMSG => "PRIVMSG",
                &Command::NOTICE => "NOTICE",
                &Command::WHO => "WHO",
                &Command::WHOIS => "WHOIS",
                &Command::WHOWAS => "WHOWAS",
                &Command::KILL => "KILL",
                &Command::PING => "PING",
                &Command::PONG => "PONG",
                &Command::ERROR => "ERROR",
                &Command::AWAY => "AWAY",
                &Command::REHASH => "REHASH",
                &Command::RESTART => "RESTART",
                &Command::SUMMON => "SUMMON",
                &Command::USERS => "USERS",
                &Command::WALLOPS => "WALLOPS",
                &Command::USERHOST => "USERHOST",
                &Command::ISON => "ISON",
            }
        )
    }
}

impl str::FromStr for Command {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "NICK" => Ok(Command::NICK),
            "PASS" => Ok(Command::PASS),
            "USER" => Ok(Command::USER),
            "SERVER" => Ok(Command::SERVER),
            "OPER" => Ok(Command::OPER),
            "QUIT" => Ok(Command::QUIT),
            "SQUIT" => Ok(Command::SQUIT),
            "JOIN" => Ok(Command::JOIN),
            "PART" => Ok(Command::PART),
            "MODE" => Ok(Command::MODE),
            "TOPIC" => Ok(Command::TOPIC),
            "NAMES" => Ok(Command::NAMES),
            "LIST" => Ok(Command::LIST),
            "INVITE" => Ok(Command::INVITE),
            "KICK" => Ok(Command::KICK),
            "VERSION" => Ok(Command::VERSION),
            "STATS" => Ok(Command::STATS),
            "LINKS" => Ok(Command::LINKS),
            "TIME" => Ok(Command::TIME),
            "CONNECT" => Ok(Command::CONNECT),
            "TRACE" => Ok(Command::TRACE),
            "ADMIN" => Ok(Command::ADMIN),
            "INFO" => Ok(Command::INFO),
            "PRIVMSG" => Ok(Command::PRIVMSG),
            "NOTICE" => Ok(Command::NOTICE),
            "WHO" => Ok(Command::WHO),
            "WHOIS" => Ok(Command::WHOIS),
            "WHOWAS" => Ok(Command::WHOWAS),
            "KILL" => Ok(Command::KILL),
            "PING" => Ok(Command::PING),
            "PONG" => Ok(Command::PONG),
            "ERROR" => Ok(Command::ERROR),
            "AWAY" => Ok(Command::AWAY),
            "REHASH" => Ok(Command::REHASH),
            "RESTART" => Ok(Command::RESTART),
            "SUMMON" => Ok(Command::SUMMON),
            "USERS" => Ok(Command::USERS),
            "WALLOPS" => Ok(Command::WALLOPS),
            "USERHOST" => Ok(Command::USERHOST),
            "ISON" => Ok(Command::ISON),
            _ => Err(ParseError::new("cannot parse command string")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Message;
    use super::Command;

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
            command: Command::NICK,
            params: ["world"],
        }
    );
    test_message_pass!(
        empty_param,
        "PASS",
        Message {
            prefix: "",
            command: Command::PASS,
            params: [],
        }
    );
    test_message_pass!(
        empty_param_trailer,
        "QUIT :",
        Message {
            prefix: "",
            command: Command::QUIT,
            params: [],
        }
    );
    test_message_pass!(
        full,
        ":lazau CONNECT server server2 :server 3 5 6",
        Message {
            prefix: ":lazau",
            command: Command::CONNECT,
            params: ["server", "server2", "server 3 5 6"],
        }
    );
}
