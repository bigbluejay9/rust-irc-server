use std::str;
use std::fmt::{self, Formatter, Error as FmtError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StatsQuery {
    C,
    H,
    I,
    K,
    L,
    M,
    O,
    U,
    Y,
    UNKNOWN(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Nick {
    pub nickname: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pass {
    pub password: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct User {
    pub username: String,
    pub mode: u32,
    pub unused: String,
    pub realname: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Server {
    pub servername: String,
    pub hopcount: u64,
    pub token: u64,
    pub info: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Oper {
    pub name: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Service {
    pub nickname: String,
    pub reserved1: String,
    pub distribution: String,
    pub ty: String,
    pub reserved2: String,
    pub info: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Quit {
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Squit {
    pub server: String,
    pub comment: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JoinChannels {
    PartAll,
    Channels(Vec<String>),
    KeyedChannels(Vec<(String, String)>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Join {
    pub join: JoinChannels,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Part {
    pub channels: Vec<String>,
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Mode {
    pub target: String,
    // TODO(lazau): This needs to be fixed.
    // https://tools.ietf.org/html/rfc2812#section-3.1.5
    // https://tools.ietf.org/html/rfc2812#section-3.2.3
    pub modespec: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Topic {
    pub channel: String,
    pub topic: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Names {
    pub channels: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct List {
    pub channels: Vec<String>,
    pub elist: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Invite {
    pub nickname: String,
    pub channel: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Kick {
    pub channels: Vec<String>,
    pub users: Vec<String>,
    pub comment: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Motd {
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lusers {
    pub mask: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Version {
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stats {
    pub query: Option<StatsQuery>,
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Links {
    pub remote_server: Option<String>,
    pub server_mask: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Time {
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Connect {
    pub target: String,
    pub port: Option<u32>,
    pub remote: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Trace {
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Admin {
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Info {
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Privmsg {
    pub targets: Vec<String>,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Notice {
    pub targets: Vec<String>,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Servlist {
    pub mask: Option<String>,
    pub server_type: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Squery {
    pub servicename: String,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Who {
    pub mask: Option<String>,
    pub operators: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Whois {
    pub target: Option<String>,
    pub masks: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Whowas {
    pub nicknames: Vec<String>,
    pub max: Option<i64>,
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Kill {
    pub nickname: String,
    pub comment: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ping {
    pub originator: String,
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pong {
    pub originator: String,
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Away {
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rehash {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Restart {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Summon {
    pub user: String,
    pub target: Option<String>,
    pub channel: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Users {
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Wallops {
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Userhost {
    pub nicknames: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ison {
    pub nicknames: Vec<String>,
}

impl str::FromStr for StatsQuery {
    type Err = super::super::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "C" => Ok(StatsQuery::C),
            "H" => Ok(StatsQuery::H),
            "I" => Ok(StatsQuery::I),
            "K" => Ok(StatsQuery::K),
            "L" => Ok(StatsQuery::L),
            "M" => Ok(StatsQuery::M),
            "O" => Ok(StatsQuery::O),
            "U" => Ok(StatsQuery::U),
            "Y" => Ok(StatsQuery::Y),
            u @ _ => Ok(StatsQuery::UNKNOWN(u.to_string())),
        }
    }
}

impl fmt::Display for Nick {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "NICK");
        unimplemented!()
    }
}

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "PASS");
        unimplemented!()
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "USER");
        unimplemented!()
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "SERVER");
        unimplemented!()
    }
}

impl fmt::Display for Oper {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "OPER");
        unimplemented!()
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "SERVICE");
        unimplemented!()
    }
}

impl fmt::Display for Quit {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "QUIT");
        unimplemented!()
    }
}

impl fmt::Display for Squit {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "SQUIT");
        unimplemented!()
    }
}

impl fmt::Display for Join {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "JOIN");
        unimplemented!()
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "PART");
        unimplemented!()
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "MODE");
        unimplemented!()
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "TOPIC");
        unimplemented!()
    }
}

impl fmt::Display for Names {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "NAMES");
        unimplemented!()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "LIST");
        unimplemented!()
    }
}

impl fmt::Display for Invite {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "INVITE");
        unimplemented!()
    }
}

impl fmt::Display for Kick {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "KICK");
        unimplemented!()
    }
}

impl fmt::Display for Motd {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "MOTD");
        unimplemented!()
    }
}

impl fmt::Display for Lusers {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "LUSERS");
        unimplemented!()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "VERSION");
        unimplemented!()
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "STATS");
        unimplemented!()
    }
}

impl fmt::Display for Links {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "LINKS");
        unimplemented!()
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "TIME");
        unimplemented!()
    }
}

impl fmt::Display for Connect {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "CONNECT");
        unimplemented!()
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "TRACE");
        unimplemented!()
    }
}

impl fmt::Display for Admin {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "ADMIN");
        unimplemented!()
    }
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "INFO");
        unimplemented!()
    }
}

impl fmt::Display for Privmsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "PRIVMSG");
        unimplemented!()
    }
}

impl fmt::Display for Notice {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "NOTICE");
        unimplemented!()
    }
}

impl fmt::Display for Servlist {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "SERVLIST");
        unimplemented!()
    }
}

impl fmt::Display for Squery {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "SQUERY");
        unimplemented!()
    }
}

impl fmt::Display for Who {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "WHO");
        unimplemented!()
    }
}

impl fmt::Display for Whois {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "WHOIS");
        unimplemented!()
    }
}

impl fmt::Display for Whowas {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "WHOWAS");
        unimplemented!()
    }
}

impl fmt::Display for Kill {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "KILL");
        unimplemented!()
    }
}

impl fmt::Display for Ping {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "PING");
        unimplemented!()
    }
}

impl fmt::Display for Pong {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "PONG");
        unimplemented!()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "ERROR");
        unimplemented!()
    }
}

impl fmt::Display for Away {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "AWAY");
        unimplemented!()
    }
}

impl fmt::Display for Rehash {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "REHASH");
        unimplemented!()
    }
}

impl fmt::Display for Restart {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "RESTART");
        unimplemented!()
    }
}

impl fmt::Display for Summon {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "SUMMON");
        unimplemented!()
    }
}

impl fmt::Display for Users {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "USERS");
        unimplemented!()
    }
}

impl fmt::Display for Wallops {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "WALLOPS");
        unimplemented!()
    }
}

impl fmt::Display for Userhost {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "USERHOST");
        unimplemented!()
    }
}

impl fmt::Display for Ison {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "ISON");
        unimplemented!()
    }
}
