mod serializer;
mod parser;

pub use self::serializer::{to_string, Error as SerializerError};
pub use self::parser::ParseError;

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Message {
    #[serde(serialize_with = "serializer::message_prefix_serializer")]
    pub prefix: Option<String>,
    pub command: Command,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(untagged)]
pub enum Command {
    Req(Request),
    Resp(Response),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct UserModes {
    //modes: HashSet<UserMode>,
    modes: UserMode,
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
pub struct ChannelModes {
    //modes: HashSet<ChannelMode>,
    modes: ChannelMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum ChannelMode {
    I,
    S,
    W,
    O,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum ModeModifier {
    Add,
    Sub,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum JoinChannels {
    Channels(Vec<String>),
    KeyedChannels(Vec<(String, String)>),
    PartAll,
}

// RFC 1459 4, 5. RFC 2812.
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
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
    SERVICE {
        nickname: String,
        reserved1: String,
        distribution: String,
        ty: String,
        reserved2: String,
        info: String,
    },
    QUIT { message: Option<String> },
    SQUIT { server: String, comment: String },

    // 4.2 Channel Operations.
    #[serde(serialize_with(serialize_with = "serializer::join_serializer")]
    JOIN { channels: JoinChannels },
    PART {
        channels: Vec<String>,
        message: Option<String>,
    },
    // TODO(lazau): Verify.
    MODE {
        target: String,
        mode: Option<RequestedMode>,
    },
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
        channels: Vec<String>,
        users: Vec<String>,
        comment: Option<String>,
    },

    // 4.3 Server queries and commands.
    // RFC 2812 additions.
    MOTD { target: Option<String> },
    LUSERS {
        mask: Option<String>,
        target: Option<String>,
    },
    // END RFC 2812 additions.
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
        port: Option<u32>,
        remote: Option<String>,
    },
    TRACE { target: Option<String> },
    ADMIN { target: Option<String> },
    INFO { target: Option<String> },

    // 4.4 Sending messages.
    PRIVMSG {
        targets: Vec<String>,
        message: String,
    },
    NOTICE {
        targets: Vec<String>,
        message: String,
    },

    // 4.5 User based queries.
    // RFC 2812 additions (Service Query and Commands).
    SERVLIST {
        mask: Option<String>,
        server_type: Option<String>,
    },
    SQUERY { servicename: String, text: String },
    // END RFC 2812 additions.
    WHO {
        mask: Option<String>,
        operators: bool,
    },
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
        originator: String,
        target: Option<String>,
    },
    PONG {
        originator: String,
        target: Option<String>,
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
    WALLOPS { message: String },
    USERHOST { nicknames: Vec<String> },
    ISON { nicknames: Vec<String> },
}

// RFC 1459 6
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
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
