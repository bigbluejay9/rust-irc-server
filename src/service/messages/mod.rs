// mod responses;
// mod requests;

// pub use self::responses::Response;
// pub use self::requests::Request;

use std;
use std::collections::HashSet;
use std::fmt::{self, Write};
use std::str;

use serde;
use serde::ser::{self, Serialize};

#[derive(Debug)]
pub enum ParseErrorKind {
    NoCommand,
    UnrecognizedCommand,
    NeedMoreParams,
    TooManyParams,
    ParseIntError,
    Other,
}

#[derive(Debug)]
pub struct ParseError {
    desc: &'static str,
    kind: ParseErrorKind,
}

fn message_prefix_serializer<S>(t: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    match t {
        &Some(ref prefix) => serializer.serialize_str(&format!(":{} ", prefix)),
        // TODO(lazau): Figure out how to return S::Ok.
        // For not rely on the fact that serializing unit produces nothing.
        &None => serializer.serialize_unit(),
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(serialize_with = "message_prefix_serializer")]
    pub prefix: Option<String>,
    pub command: Command,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Command {
    Req(Request),
    Resp(Response),
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

impl serde::ser::Error for ParseError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        unimplemented!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserModes {
    //modes: HashSet<UserMode>,
    modes: UserMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ChannelModes {
    //modes: HashSet<ChannelMode>,
    modes: ChannelMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ChannelMode {
    I,
    S,
    W,
    O,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ModeModifier {
    Add,
    Sub,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum StatsQuery {
    C,
    H,
    I,
    K,
    L,
    M,
    O,
    Y,
    U,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
enum JoinChannels {
    Channels(Vec<String>),
    KeyedChannels(Vec<(String, String)>),
}

// RFC 1459 4, 5. RFC 2812.
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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

// RFC 1459 6
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Response {
    // 6.1 Error replies.
    #[serde(rename = "401")]
    ERR_NOSUCHNICK,
    #[serde(rename = "402")]
    ERR_NOSUCHSERVER,
    #[serde(rename = "403")]
    ERR_NOSUCHCHANNEL,
    #[serde(rename = "404")]
    ERR_CANNOTSENDTOCHAN,
    #[serde(rename = "405")]
    ERR_TOOMANYCHANNELS,
    #[serde(rename = "406")]
    ERR_WASNOSUCHNICK,
    #[serde(rename = "407")]
    ERR_TOOMANYTARGETS,
    #[serde(rename = "409")]
    ERR_NOORIGIN,
    #[serde(rename = "411")]
    ERR_NORECIPIENT,
    #[serde(rename = "412")]
    ERR_NOTEXTTOSEND,
    #[serde(rename = "413")]
    ERR_NOTOPLEVEL,
    #[serde(rename = "414")]
    ERR_WILDTOPLEVEL,
    #[serde(rename = "421")]
    ERR_UNKNOWNCOMMAND,
    #[serde(rename = "422")]
    ERR_NOMOTD,
    #[serde(rename = "423")]
    ERR_NOADMININFO,
    #[serde(rename = "424")]
    ERR_FILEERROR,
    #[serde(rename = "431")]
    ERR_NONICKNAMEGIVEN,
    #[serde(rename = "432")]
    ERR_ERRONEUSNICKNAME,
    #[serde(rename = "433")]
    ERR_NICKNAMEINUSE,
    #[serde(rename = "436")]
    ERR_NICKCOLLISION,
    #[serde(rename = "441")]
    ERR_USERNOTINCHANNEL,
    #[serde(rename = "442")]
    ERR_NOTONCHANNEL,
    #[serde(rename = "443")]
    ERR_USERONCHANNEL,
    #[serde(rename = "444")]
    ERR_NOLOGIN,
    #[serde(rename = "445")]
    ERR_SUMMONDISABLED,
    #[serde(rename = "446")]
    ERR_USERSDISABLED,
    #[serde(rename = "451")]
    ERR_NOTREGISTERED,
    #[serde(rename = "461")]
    ERR_NEEDMOREPARAMS,
    #[serde(rename = "462")]
    ERR_ALREADYREGISTRED,
    #[serde(rename = "463")]
    ERR_NOPERMFORHOST,
    #[serde(rename = "464")]
    ERR_PASSWDMISMATCH,
    #[serde(rename = "465")]
    ERR_YOUREBANNEDCREEP,
    #[serde(rename = "467")]
    ERR_KEYSET,
    #[serde(rename = "471")]
    ERR_CHANNELISFULL,
    #[serde(rename = "472")]
    ERR_UNKNOWNMODE,
    #[serde(rename = "473")]
    ERR_INVITEONLYCHAN,
    #[serde(rename = "474")]
    ERR_BANNEDFROMCHAN,
    #[serde(rename = "475")]
    ERR_BADCHANNELKEY,
    #[serde(rename = "481")]
    ERR_NOPRIVILEGES,
    #[serde(rename = "482")]
    ERR_CHANOPRIVSNEEDED,
    #[serde(rename = "483")]
    ERR_CANTKILLSERVER,
    #[serde(rename = "491")]
    ERR_NOOPERHOST,
    #[serde(rename = "501")]
    ERR_UMODEUNKNOWNFLAG,
    #[serde(rename = "502")]
    ERR_USERSDONTMATCH,

    // 6.2 Command responses.
    #[serde(rename = "300")]
    RPL_NONE,
    #[serde(rename = "302")]
    RPL_USERHOST,
    #[serde(rename = "303")]
    RPL_ISON,
    #[serde(rename = "301")]
    RPL_AWAY,
    #[serde(rename = "305")]
    RPL_UNAWAY,
    #[serde(rename = "306")]
    RPL_NOWAWAY,
    #[serde(rename = "311")]
    RPL_WHOISUSER,
    #[serde(rename = "312")]
    RPL_WHOISSERVER,
    #[serde(rename = "313")]
    RPL_WHOISOPERATOR,
    #[serde(rename = "317")]
    RPL_WHOISIDLE,
    #[serde(rename = "318")]
    RPL_ENDOFWHOIS,
    #[serde(rename = "319")]
    RPL_WHOISCHANNELS,
    #[serde(rename = "314")]
    RPL_WHOWASUSER,
    #[serde(rename = "369")]
    RPL_ENDOFWHOWAS,
    #[serde(rename = "321")]
    RPL_LISTSTART,
    #[serde(rename = "322")]
    RPL_LIST,
    #[serde(rename = "323")]
    RPL_LISTEND,
    #[serde(rename = "324")]
    RPL_CHANNELMODEIS,
    #[serde(rename = "331")]
    RPL_NOTOPIC,
    #[serde(rename = "332")]
    RPL_TOPIC,
    #[serde(rename = "341")]
    RPL_INVITING,
    #[serde(rename = "342")]
    RPL_SUMMONING,
    #[serde(rename = "351")]
    RPL_VERSION,
    #[serde(rename = "352")]
    RPL_WHOREPLY,
    #[serde(rename = "315")]
    RPL_ENDOFWHO,
    #[serde(rename = "353")]
    RPL_NAMREPLY,
    #[serde(rename = "366")]
    RPL_ENDOFNAMES,
    #[serde(rename = "364")]
    RPL_LINKS,
    #[serde(rename = "365")]
    RPL_ENDOFLINKS,
    #[serde(rename = "367")]
    RPL_BANLIST,
    #[serde(rename = "368")]
    RPL_ENDOFBANLIST,
    #[serde(rename = "371")]
    RPL_INFO,
    #[serde(rename = "374")]
    RPL_ENDOFINFO,
    #[serde(rename = "375")]
    RPL_MOTDSTART,
    #[serde(rename = "372")]
    RPL_MOTD,
    #[serde(rename = "376")]
    RPL_ENDOFMOTD,
    #[serde(rename = "381")]
    RPL_YOUREOPER,
    #[serde(rename = "382")]
    RPL_REHASHING,
    #[serde(rename = "391")]
    RPL_TIME,
    #[serde(rename = "392")]
    RPL_USERSSTART,
    #[serde(rename = "393")]
    RPL_USERS,
    #[serde(rename = "394")]
    RPL_ENDOFUSERS,
    #[serde(rename = "395")]
    RPL_NOUSERS,
    #[serde(rename = "200")]
    RPL_TRACELINK,
    #[serde(rename = "201")]
    RPL_TRACECONNECTING,
    #[serde(rename = "202")]
    RPL_TRACEHANDSHAKE,
    #[serde(rename = "203")]
    RPL_TRACEUNKNOWN,
    #[serde(rename = "204")]
    RPL_TRACEOPERATOR,
    #[serde(rename = "205")]
    RPL_TRACEUSER,
    #[serde(rename = "206")]
    RPL_TRACESERVER,
    #[serde(rename = "208")]
    RPL_TRACENEWTYPE,
    #[serde(rename = "261")]
    RPL_TRACELOG,
    #[serde(rename = "211")]
    RPL_STATSLINKINFO,
    #[serde(rename = "212")]
    RPL_STATSCOMMANDS,
    #[serde(rename = "213")]
    RPL_STATSCLINE,
    #[serde(rename = "214")]
    RPL_STATSNLINE,
    #[serde(rename = "215")]
    RPL_STATSILINE,
    #[serde(rename = "216")]
    RPL_STATSKLINE,
    #[serde(rename = "218")]
    RPL_STATSYLINE,
    #[serde(rename = "219")]
    RPL_ENDOFSTATS,
    #[serde(rename = "241")]
    RPL_STATSLLINE,
    #[serde(rename = "242")]
    RPL_STATSUPTIME,
    #[serde(rename = "243")]
    RPL_STATSOLINE,
    #[serde(rename = "244")]
    RPL_STATSHLINE,
    #[serde(rename = "221")]
    RPL_UMODEIS,
    #[serde(rename = "251")]
    RPL_LUSERCLIENT,
    #[serde(rename = "252")]
    RPL_LUSEROP,
    #[serde(rename = "253")]
    RPL_LUSERUNKNOWN,
    #[serde(rename = "254")]
    RPL_LUSERCHANNELS,
    #[serde(rename = "255")]
    RPL_LUSERME,
    #[serde(rename = "256")]
    RPL_ADMINME,
    #[serde(rename = "257")]
    RPL_ADMINLOC1,
    #[serde(rename = "258")]
    RPL_ADMINLOC2,
    #[serde(rename = "259")]
    RPL_ADMINEMAIL,

    // 6.3 Reserved.
    #[serde(rename = "209")]
    RPL_TRACECLASS,
    #[serde(rename = "217")]
    RPL_STATSQLINE,
    #[serde(rename = "231")]
    RPL_SERVICEINFO,
    #[serde(rename = "232")]
    RPL_ENDOFSERVICES,
    #[serde(rename = "233")]
    RPL_SERVICE,
    #[serde(rename = "234")]
    RPL_SERVLIST,
    #[serde(rename = "235")]
    RPL_SERVLISTEND,
    #[serde(rename = "316")]
    RPL_WHOISCHANOP,
    #[serde(rename = "361")]
    RPL_KILLDONE,
    #[serde(rename = "362")]
    RPL_CLOSING,
    #[serde(rename = "363")]
    RPL_CLOSEEND,
    #[serde(rename = "373")]
    RPL_INFOSTART,
    #[serde(rename = "384")]
    RPL_MYPORTIS,
    #[serde(rename = "466")]
    ERR_YOUWILLBEBANNED,
    #[serde(rename = "476")]
    ERR_BADCHANMASK,
    #[serde(rename = "492")]
    ERR_NOSERVICEHOST,

    // RFC 2812 5.1 Command responses.
    #[serde(rename = "001")]
    RPL_WELCOME { message: Option<String> },
    #[serde(rename = "002")]
    RPL_YOURHOST,
    #[serde(rename = "003")]
    RPL_CREATED,
    #[serde(rename = "004")]
    RPL_MYINFO,
    #[serde(rename = "005")]
    RPL_ISUPPORT,
    #[serde(rename = "010")]
    RPL_BOUNCE,
}

#[derive(Default, Debug)]
pub struct IRCSerializer {
    output: String,
    last_colon: Option<usize>,
}

pub fn to_string<T>(value: &T) -> std::result::Result<String, ParseError>
where
    T: Serialize,
{
    let mut serializer = IRCSerializer::default(); //{ output: String::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.output += str::from_utf8(v).unwrap();
        Ok(())
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // Don't actually write anything.
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        println!(
            "name; {} variant_index: {} variatn: {}",
            name,
            variant_index,
            variant
        );
        unimplemented!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.output += &format!("{}", variant);
        Ok(self)
    }
}


impl<'a> ser::SerializeSeq for &'a mut IRCSerializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = ParseError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with(':') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;

    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;

    fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;

    fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;

    fn serialize_field<T>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let initial_len = self.output.as_bytes().len();
        self.output += " :";
        let mut new_colon_location = self.output.as_bytes().len() - 1;
        let after_space_len = self.output.as_bytes().len();
        value.serialize(&mut **self)?;
        // If value produced a string, remove the last colon, otherwise trim the " :" string added.
        if after_space_len != self.output.as_bytes().len() {
            if let Some(last_colon_pos) = self.last_colon {
                new_colon_location -= 1;
                unsafe {
                    self.output.as_mut_vec().remove(last_colon_pos);
                }
            }
            self.last_colon = Some(new_colon_location);
        } else {
            // Trim the added space.
            unsafe {
                self.output.as_mut_vec().truncate(initial_len);
            }
        }
        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::{to_string, Message, Command, Request, Response};

    macro_rules! verify_serialize{
        ($serialized:expr, $message:expr) => {
            assert_eq!($serialized.to_string(), to_string(&$message).unwrap());
        }
    }

    #[test]
    fn test_serialize() {
        verify_serialize!(
            "NICK :lazau",
            Message {
                prefix: None,
                command: Command::Req(Request::NICK { nickname: "lazau".to_string() }),
            }
        );

        verify_serialize!(
            "QUIT",
            Message {
                prefix: None,
                command: Command::Req(Request::QUIT { message: None }),
            }
        );

        verify_serialize!(
            "QUIT :quit_message",
            Message {
                prefix: None,
                command: Command::Req(Request::QUIT { message: Some("quit_message".to_string()) }),
            }
        );

        verify_serialize!(
            "PART :channel_a,channel_b,channel_c",
            Message {
                prefix: None,
                command: Command::Req(Request::PART {
                    channels: vec![
                        "channel_a".to_string(),
                        "channel_b".to_string(),
                        "channel_c".to_string(),
                    ],
                    message: None,
                }),
            }
        );

        verify_serialize!(
            ":irc.mozilla.org PART channel_11 :A parting message",
            Message {
                prefix: Some("irc.mozilla.org".to_string()),
                command: Command::Req(Request::PART {
                    channels: vec!["channel_11".to_string()],
                    message: Some("A parting message".to_string()),
                }),
            }
        );

        verify_serialize!(
            ":WiZ CONNECT eff.org 6667 :csd.bu.edu",
            Message {
                prefix: Some("WiZ".to_string()),
                command: Command::Req(Request::CONNECT {
                    target: "eff.org".to_string(),
                    port: 6667,
                    remote: Some("csd.bu.edu".to_string()),
                }),
            }
        );

        verify_serialize!(
            ":irc.freenode.net 461",
            Message {
                prefix: Some("irc.freenode.net".to_string()),
                command: Command::Resp(Response::ERR_NEEDMOREPARAMS),
            }
        );

        verify_serialize!(
            ":irc.freenode.net 001",
            Message {
                prefix: Some("irc.freenode.net".to_string()),
                command: Command::Resp(Response::RPL_WELCOME { message: None }),
            }
        );

        verify_serialize!(
            ":irc.freenode.net 001 :Welcome to the network, friend!",
            Message {
                prefix: Some("irc.freenode.net".to_string()),
                command: Command::Resp(Response::RPL_WELCOME {
                    message: Some("Welcome to the network, friend!".to_string()),
                }),
            }
        );
    }
}
