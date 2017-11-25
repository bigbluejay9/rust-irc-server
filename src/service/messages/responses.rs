use chrono;

use std::fmt;
use std::str;

/*// Translates Enum val to builder Type.
fn builder_for(r: &Response) -> Box<super::ParamBuilder> {
    match r {
        &Response::ERR_NONICKNAMEGIVEN => Box::new(NoNicknameGivenParamsBuilder::default()),
        &Response::ERR_NICKNAMEINUSE => Box::new(NickNameInUseParamsBuilder::default()),
        &Response::ERR_NEEDMOREPARAMS => Box::new(NeedMoreParamsParamsBuilder::default()),
        &Response::RPL_WELCOME => Box::new(WelcomeParamsBuilder::default()),
        &Response::RPL_YOURHOST => Box::new(YourHostParamsBuilder::default()),
        &Response::RPL_CREATED => Box::new(CreatedParamsBuilder::default()),
        &Response::RPL_MYINFO => Box::new(MyInfoParamsBuilder::default()),
        _ => unimplemented!(),
    }
}

#[derive(Default)]
pub struct NoNicknameGivenParamsBuilder;

impl super::ParamBuilder for NoNicknameGivenParamsBuilder {
    fn build_params(self) -> Vec<String> {
        vec!["No nickname given".to_string()]
    }
}

#[derive(Default)]
pub struct NickNameInUseParamsBuilder;

impl super::ParamBuilder for NickNameInUseParamsBuilder {
    fn build_params(self) -> Vec<String> {
        vec!["Nickname is already in use".to_string()]
    }
}

#[derive(Default)]
pub struct NeedMoreParamsParamsBuilder;

impl super::ParamBuilder for NeedMoreParamsParamsBuilder {
    fn build_params(self) -> Vec<String> {
        vec!["Not enough parameters".to_string()]
    }
}

#[derive(Default)]
pub struct WelcomeParamsBuilder<'a> {
    nick: Option<&'a String>,
    network_name: Option<&'a String>,
    user: Option<&'a String>,
    host: Option<&'a String>,
}

impl<'a> super::ParamBuilder for WelcomeParamsBuilder<'a> {
    fn build_params(self) -> Vec<String> {
        let nick = self.nick.expect(
            "nick/client param required for RPL_WELCOME",
        );
        let mut params = vec![format!("{}", nick)];
        let mut last_param = format!("Welcome to <networkname> Network, {}", nick);
        if let (Some(u), Some(h)) = (self.user, self.host) {
            last_param = format!("{}[!{}@{:?}]", nick, u, h);
        }
        params.push(last_param);
        params
    }
}

impl<'a> WelcomeParamsBuilder<'a> {
    pub fn with_nick(&mut self, nick: &'a String) -> &mut Self {
        self.nick = Some(nick);
        self
    }

    pub fn with_client(&mut self, c: &'a String) -> &mut Self {
        self.with_nick(c)
    }

    pub fn with_network_name(&mut self, nn: &'a String) -> &mut Self {
        self.network_name = Some(nn);
        self
    }

    pub fn with_user_and_host(&mut self, user: &'a String, host: &'a String) -> &mut Self {
        self.user = Some(user);
        self.host = Some(host);
        self
    }
}

#[derive(Default)]
pub struct YourHostParamsBuilder<'a> {
    client: Option<&'a String>,
    server_name: Option<&'a String>,
    version: Option<&'a String>,
}

impl<'a> super::ParamBuilder for YourHostParamsBuilder<'a> {
    fn build_params(self) -> Vec<String> {
        vec![
            format!("{}", self.client.expect("client for RPL_YOURHOST")),
            format!(
                "Your host is {}, running version {}",
                self.server_name.expect("server_name for RPL_YOURHOST"),
                self.version.expect("version for RPL_YOURHOST")
            ),
        ]
    }
}

impl<'a> YourHostParamsBuilder<'a> {
    pub fn with_client(&mut self, client: &'a String) -> &mut Self {
        self.client = Some(client);
        self
    }

    pub fn with_server_name(&mut self, sn: &'a String) -> &mut Self {
        self.server_name = Some(sn);
        self
    }

    pub fn with_version(&mut self, version: &'a String) -> &mut Self {
        self.version = Some(version);
        self
    }
}

#[derive(Default)]
pub struct CreatedParamsBuilder<'a> {
    client: Option<&'a String>,
    date_time: Option<&'a chrono::DateTime<chrono::Utc>>,
}

impl<'a> super::ParamBuilder for CreatedParamsBuilder<'a> {
    fn build_params(self) -> Vec<String> {
        vec![
            format!("{}", self.client.expect("client for RPL_CREATED")),
            format!(
                "This server was created {}",
                self.date_time
                    .expect("date_time for RPL_CREATED")
                    .to_rfc2822()
            ),
        ]
    }
}

impl<'a> CreatedParamsBuilder<'a> {
    pub fn with_client(&mut self, client: &'a String) -> &mut Self {
        self.client = Some(client);
        self
    }

    pub fn with_date_time(&mut self, date_time: &'a chrono::DateTime<chrono::Utc>) -> &mut Self {
        self.date_time = Some(date_time);
        self
    }
}

#[derive(Default)]
pub struct MyInfoParamsBuilder<'a> {
    client: Option<&'a String>,
    server_name: Option<&'a String>,
    version: Option<&'a String>,
    user_modes: Option<&'a String>,
    channel_modes: Option<&'a String>,
    // TODO(lazau): channel mode with parameter??
}

impl<'a> super::ParamBuilder for MyInfoParamsBuilder<'a> {
    fn build_params(self) -> Vec<String> {
        vec![
            format!("{}", self.client.expect("client for RPL_MYINFO")),
            format!("{}", self.server_name.expect("server_name for RPL_MYINFO")),
            format!("{}", self.version.expect("version for RPL_MYINFO")),
            format!("{}", self.user_modes.expect("user_modes for RPL_MYINFO")),
            format!(
                "{}",
                self.channel_modes.expect("channel_modes for RPL_MYINFO")
            ),
        ]
    }
}

impl<'a> MyInfoParamsBuilder<'a> {
    pub fn with_client(&mut self, client: &'a String) -> &mut Self {
        self.client = Some(client);
        self
    }

    pub fn with_server_name(&mut self, sn: &'a String) -> &mut Self {
        self.server_name = Some(sn);
        self
    }

    pub fn with_version(&mut self, version: &'a String) -> &mut Self {
        self.version = Some(version);
        self
    }

    pub fn with_user_modes(&mut self, user_modes: &'a String) -> &mut Self {
        self.user_modes = Some(user_modes);
        self
    }

    pub fn with_channel_modes(&mut self, channel_modes: &'a String) -> &mut Self {
        self.channel_modes = Some(channel_modes);
        self
    }
}*/

// RFC 1459 6
#[allow(non_camel_case_types)]
#[derive(PartialEq, Clone)]
pub enum Response {
    // 6.1 Error replies.
    ERR_NOSUCHNICK = 401,
    ERR_NOSUCHSERVER = 402,
    ERR_NOSUCHCHANNEL = 403,
    ERR_CANNOTSENDTOCHAN = 404,
    ERR_TOOMANYCHANNELS = 405,
    ERR_WASNOSUCHNICK = 406,
    ERR_TOOMANYTARGETS = 407,
    ERR_NOORIGIN = 409,
    ERR_NORECIPIENT = 411,
    ERR_NOTEXTTOSEND = 412,
    ERR_NOTOPLEVEL = 413,
    ERR_WILDTOPLEVEL = 414,
    ERR_UNKNOWNCOMMAND = 421,
    ERR_NOMOTD = 422,
    ERR_NOADMININFO = 423,
    ERR_FILEERROR = 424,
    ERR_NONICKNAMEGIVEN = 431,
    ERR_ERRONEUSNICKNAME = 432,
    ERR_NICKNAMEINUSE = 433,
    ERR_NICKCOLLISION = 436,
    ERR_USERNOTINCHANNEL = 441,
    ERR_NOTONCHANNEL = 442,
    ERR_USERONCHANNEL = 443,
    ERR_NOLOGIN = 444,
    ERR_SUMMONDISABLED = 445,
    ERR_USERSDISABLED = 446,
    ERR_NOTREGISTERED = 451,
    ERR_NEEDMOREPARAMS = 461,
    ERR_ALREADYREGISTRED = 462,
    ERR_NOPERMFORHOST = 463,
    ERR_PASSWDMISMATCH = 464,
    ERR_YOUREBANNEDCREEP = 465,
    ERR_KEYSET = 467,
    ERR_CHANNELISFULL = 471,
    ERR_UNKNOWNMODE = 472,
    ERR_INVITEONLYCHAN = 473,
    ERR_BANNEDFROMCHAN = 474,
    ERR_BADCHANNELKEY = 475,
    ERR_NOPRIVILEGES = 481,
    ERR_CHANOPRIVSNEEDED = 482,
    ERR_CANTKILLSERVER = 483,
    ERR_NOOPERHOST = 491,
    ERR_UMODEUNKNOWNFLAG = 501,
    ERR_USERSDONTMATCH = 502,

    // 6.2 Command responses.
    RPL_NONE = 300,
    RPL_USERHOST = 302,
    RPL_ISON = 303,
    RPL_AWAY = 301,
    RPL_UNAWAY = 305,
    RPL_NOWAWAY = 306,
    RPL_WHOISUSER = 311,
    RPL_WHOISSERVER = 312,
    RPL_WHOISOPERATOR = 313,
    RPL_WHOISIDLE = 317,
    RPL_ENDOFWHOIS = 318,
    RPL_WHOISCHANNELS = 319,
    RPL_WHOWASUSER = 314,
    RPL_ENDOFWHOWAS = 369,
    RPL_LISTSTART = 321,
    RPL_LIST = 322,
    RPL_LISTEND = 323,
    RPL_CHANNELMODEIS = 324,
    RPL_NOTOPIC = 331,
    RPL_TOPIC = 332,
    RPL_INVITING = 341,
    RPL_SUMMONING = 342,
    RPL_VERSION = 351,
    RPL_WHOREPLY = 352,
    RPL_ENDOFWHO = 315,
    RPL_NAMREPLY = 353,
    RPL_ENDOFNAMES = 366,
    RPL_LINKS = 364,
    RPL_ENDOFLINKS = 365,
    RPL_BANLIST = 367,
    RPL_ENDOFBANLIST = 368,
    RPL_INFO = 371,
    RPL_ENDOFINFO = 374,
    RPL_MOTDSTART = 375,
    RPL_MOTD = 372,
    RPL_ENDOFMOTD = 376,
    RPL_YOUREOPER = 381,
    RPL_REHASHING = 382,
    RPL_TIME = 391,
    RPL_USERSSTART = 392,
    RPL_USERS = 393,
    RPL_ENDOFUSERS = 394,
    RPL_NOUSERS = 395,
    RPL_TRACELINK = 200,
    RPL_TRACECONNECTING = 201,
    RPL_TRACEHANDSHAKE = 202,
    RPL_TRACEUNKNOWN = 203,
    RPL_TRACEOPERATOR = 204,
    RPL_TRACEUSER = 205,
    RPL_TRACESERVER = 206,
    RPL_TRACENEWTYPE = 208,
    RPL_TRACELOG = 261,
    RPL_STATSLINKINFO = 211,
    RPL_STATSCOMMANDS = 212,
    RPL_STATSCLINE = 213,
    RPL_STATSNLINE = 214,
    RPL_STATSILINE = 215,
    RPL_STATSKLINE = 216,
    RPL_STATSYLINE = 218,
    RPL_ENDOFSTATS = 219,
    RPL_STATSLLINE = 241,
    RPL_STATSUPTIME = 242,
    RPL_STATSOLINE = 243,
    RPL_STATSHLINE = 244,
    RPL_UMODEIS = 221,
    RPL_LUSERCLIENT = 251,
    RPL_LUSEROP = 252,
    RPL_LUSERUNKNOWN = 253,
    RPL_LUSERCHANNELS = 254,
    RPL_LUSERME = 255,
    RPL_ADMINME = 256,
    RPL_ADMINLOC1 = 257,
    RPL_ADMINLOC2 = 258,
    RPL_ADMINEMAIL = 259,

    // 6.3 Reserved.
    RPL_TRACECLASS = 209,
    RPL_STATSQLINE = 217,
    RPL_SERVICEINFO = 231,
    RPL_ENDOFSERVICES = 232,
    RPL_SERVICE = 233,
    RPL_SERVLIST = 234,
    RPL_SERVLISTEND = 235,
    RPL_WHOISCHANOP = 316,
    RPL_KILLDONE = 361,
    RPL_CLOSING = 362,
    RPL_CLOSEEND = 363,
    RPL_INFOSTART = 373,
    RPL_MYPORTIS = 384,
    ERR_YOUWILLBEBANNED = 466,
    ERR_BADCHANMASK = 476,
    ERR_NOSERVICEHOST = 492,

    // RFC 2812 5.1 Command responses.
    RPL_WELCOME = 1,
    RPL_YOURHOST = 2,
    RPL_CREATED = 3,
    RPL_MYINFO = 4,
    RPL_BOUNCE = 5,
}

impl Response {
    pub fn build_params(&self) -> Vec<String> {
        Vec::new()
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Response::ERR_NOSUCHNICK => "401",
                &Response::ERR_NOSUCHSERVER => "402",
                &Response::ERR_NOSUCHCHANNEL => "403",
                &Response::ERR_CANNOTSENDTOCHAN => "404",
                &Response::ERR_TOOMANYCHANNELS => "405",
                &Response::ERR_WASNOSUCHNICK => "406",
                &Response::ERR_TOOMANYTARGETS => "407",
                &Response::ERR_NOORIGIN => "409",
                &Response::ERR_NORECIPIENT => "411",
                &Response::ERR_NOTEXTTOSEND => "412",
                &Response::ERR_NOTOPLEVEL => "413",
                &Response::ERR_WILDTOPLEVEL => "414",
                &Response::ERR_UNKNOWNCOMMAND => "421",
                &Response::ERR_NOMOTD => "422",
                &Response::ERR_NOADMININFO => "423",
                &Response::ERR_FILEERROR => "424",
                &Response::ERR_NONICKNAMEGIVEN => "431",
                &Response::ERR_ERRONEUSNICKNAME => "432",
                &Response::ERR_NICKNAMEINUSE => "433",
                &Response::ERR_NICKCOLLISION => "436",
                &Response::ERR_USERNOTINCHANNEL => "441",
                &Response::ERR_NOTONCHANNEL => "442",
                &Response::ERR_USERONCHANNEL => "443",
                &Response::ERR_NOLOGIN => "444",
                &Response::ERR_SUMMONDISABLED => "445",
                &Response::ERR_USERSDISABLED => "446",
                &Response::ERR_NOTREGISTERED => "451",
                &Response::ERR_NEEDMOREPARAMS => "461",
                &Response::ERR_ALREADYREGISTRED => "462",
                &Response::ERR_NOPERMFORHOST => "463",
                &Response::ERR_PASSWDMISMATCH => "464",
                &Response::ERR_YOUREBANNEDCREEP => "465",
                &Response::ERR_KEYSET => "467",
                &Response::ERR_CHANNELISFULL => "471",
                &Response::ERR_UNKNOWNMODE => "472",
                &Response::ERR_INVITEONLYCHAN => "473",
                &Response::ERR_BANNEDFROMCHAN => "474",
                &Response::ERR_BADCHANNELKEY => "475",
                &Response::ERR_NOPRIVILEGES => "481",
                &Response::ERR_CHANOPRIVSNEEDED => "482",
                &Response::ERR_CANTKILLSERVER => "483",
                &Response::ERR_NOOPERHOST => "491",
                &Response::ERR_UMODEUNKNOWNFLAG => "501",
                &Response::ERR_USERSDONTMATCH => "502",
                &Response::RPL_NONE => "300",
                &Response::RPL_USERHOST => "302",
                &Response::RPL_ISON => "303",
                &Response::RPL_AWAY => "301",
                &Response::RPL_UNAWAY => "305",
                &Response::RPL_NOWAWAY => "306",
                &Response::RPL_WHOISUSER => "311",
                &Response::RPL_WHOISSERVER => "312",
                &Response::RPL_WHOISOPERATOR => "313",
                &Response::RPL_WHOISIDLE => "317",
                &Response::RPL_ENDOFWHOIS => "318",
                &Response::RPL_WHOISCHANNELS => "319",
                &Response::RPL_WHOWASUSER => "314",
                &Response::RPL_ENDOFWHOWAS => "369",
                &Response::RPL_LISTSTART => "321",
                &Response::RPL_LIST => "322",
                &Response::RPL_LISTEND => "323",
                &Response::RPL_CHANNELMODEIS => "324",
                &Response::RPL_NOTOPIC => "331",
                &Response::RPL_TOPIC => "332",
                &Response::RPL_INVITING => "341",
                &Response::RPL_SUMMONING => "342",
                &Response::RPL_VERSION => "351",
                &Response::RPL_WHOREPLY => "352",
                &Response::RPL_ENDOFWHO => "315",
                &Response::RPL_NAMREPLY => "353",
                &Response::RPL_ENDOFNAMES => "366",
                &Response::RPL_LINKS => "364",
                &Response::RPL_ENDOFLINKS => "365",
                &Response::RPL_BANLIST => "367",
                &Response::RPL_ENDOFBANLIST => "368",
                &Response::RPL_INFO => "371",
                &Response::RPL_ENDOFINFO => "374",
                &Response::RPL_MOTDSTART => "375",
                &Response::RPL_MOTD => "372",
                &Response::RPL_ENDOFMOTD => "376",
                &Response::RPL_YOUREOPER => "381",
                &Response::RPL_REHASHING => "382",
                &Response::RPL_TIME => "391",
                &Response::RPL_USERSSTART => "392",
                &Response::RPL_USERS => "393",
                &Response::RPL_ENDOFUSERS => "394",
                &Response::RPL_NOUSERS => "395",
                &Response::RPL_TRACELINK => "200",
                &Response::RPL_TRACECONNECTING => "201",
                &Response::RPL_TRACEHANDSHAKE => "202",
                &Response::RPL_TRACEUNKNOWN => "203",
                &Response::RPL_TRACEOPERATOR => "204",
                &Response::RPL_TRACEUSER => "205",
                &Response::RPL_TRACESERVER => "206",
                &Response::RPL_TRACENEWTYPE => "208",
                &Response::RPL_TRACELOG => "261",
                &Response::RPL_STATSLINKINFO => "211",
                &Response::RPL_STATSCOMMANDS => "212",
                &Response::RPL_STATSCLINE => "213",
                &Response::RPL_STATSNLINE => "214",
                &Response::RPL_STATSILINE => "215",
                &Response::RPL_STATSKLINE => "216",
                &Response::RPL_STATSYLINE => "218",
                &Response::RPL_ENDOFSTATS => "219",
                &Response::RPL_STATSLLINE => "241",
                &Response::RPL_STATSUPTIME => "242",
                &Response::RPL_STATSOLINE => "243",
                &Response::RPL_STATSHLINE => "244",
                &Response::RPL_UMODEIS => "221",
                &Response::RPL_LUSERCLIENT => "251",
                &Response::RPL_LUSEROP => "252",
                &Response::RPL_LUSERUNKNOWN => "253",
                &Response::RPL_LUSERCHANNELS => "254",
                &Response::RPL_LUSERME => "255",
                &Response::RPL_ADMINME => "256",
                &Response::RPL_ADMINLOC1 => "257",
                &Response::RPL_ADMINLOC2 => "258",
                &Response::RPL_ADMINEMAIL => "259",
                &Response::RPL_TRACECLASS => "209",
                &Response::RPL_STATSQLINE => "217",
                &Response::RPL_SERVICEINFO => "231",
                &Response::RPL_ENDOFSERVICES => "232",
                &Response::RPL_SERVICE => "233",
                &Response::RPL_SERVLIST => "234",
                &Response::RPL_SERVLISTEND => "235",
                &Response::RPL_WHOISCHANOP => "316",
                &Response::RPL_KILLDONE => "361",
                &Response::RPL_CLOSING => "362",
                &Response::RPL_CLOSEEND => "363",
                &Response::RPL_INFOSTART => "373",
                &Response::RPL_MYPORTIS => "384",
                &Response::ERR_YOUWILLBEBANNED => "466",
                &Response::ERR_BADCHANMASK => "476",
                &Response::ERR_NOSERVICEHOST => "492",
                &Response::RPL_WELCOME => "001",
                &Response::RPL_YOURHOST => "002",
                &Response::RPL_CREATED => "003",
                &Response::RPL_MYINFO => "004",
                &Response::RPL_BOUNCE => "005",
            }
        )
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let o = match self {
            &Response::ERR_NOSUCHNICK => ("ERR_NOSUCHNICK", "401"),
            &Response::ERR_NOSUCHSERVER => ("ERR_NOSUCHSERVER", "402"),
            &Response::ERR_NOSUCHCHANNEL => ("ERR_NOSUCHCHANNEL", "403"),
            &Response::ERR_CANNOTSENDTOCHAN => ("ERR_CANNOTSENDTOCHAN", "404"),
            &Response::ERR_TOOMANYCHANNELS => ("ERR_TOOMANYCHANNELS", "405"),
            &Response::ERR_WASNOSUCHNICK => ("ERR_WASNOSUCHNICK", "406"),
            &Response::ERR_TOOMANYTARGETS => ("ERR_TOOMANYTARGETS", "407"),
            &Response::ERR_NOORIGIN => ("ERR_NOORIGIN", "409"),
            &Response::ERR_NORECIPIENT => ("ERR_NORECIPIENT", "411"),
            &Response::ERR_NOTEXTTOSEND => ("ERR_NOTEXTTOSEND", "412"),
            &Response::ERR_NOTOPLEVEL => ("ERR_NOTOPLEVEL", "413"),
            &Response::ERR_WILDTOPLEVEL => ("ERR_WILDTOPLEVEL", "414"),
            &Response::ERR_UNKNOWNCOMMAND => ("ERR_UNKNOWNCOMMAND", "421"),
            &Response::ERR_NOMOTD => ("ERR_NOMOTD", "422"),
            &Response::ERR_NOADMININFO => ("ERR_NOADMININFO", "423"),
            &Response::ERR_FILEERROR => ("ERR_FILEERROR", "424"),
            &Response::ERR_NONICKNAMEGIVEN => ("ERR_NONICKNAMEGIVEN", "431"),
            &Response::ERR_ERRONEUSNICKNAME => ("ERR_ERRONEUSNICKNAME", "432"),
            &Response::ERR_NICKNAMEINUSE => ("ERR_NICKNAMEINUSE", "433"),
            &Response::ERR_NICKCOLLISION => ("ERR_NICKCOLLISION", "436"),
            &Response::ERR_USERNOTINCHANNEL => ("ERR_USERNOTINCHANNEL", "441"),
            &Response::ERR_NOTONCHANNEL => ("ERR_NOTONCHANNEL", "442"),
            &Response::ERR_USERONCHANNEL => ("ERR_USERONCHANNEL", "443"),
            &Response::ERR_NOLOGIN => ("ERR_NOLOGIN", "444"),
            &Response::ERR_SUMMONDISABLED => ("ERR_SUMMONDISABLED", "445"),
            &Response::ERR_USERSDISABLED => ("ERR_USERSDISABLED", "446"),
            &Response::ERR_NOTREGISTERED => ("ERR_NOTREGISTERED", "451"),
            &Response::ERR_NEEDMOREPARAMS => ("ERR_NEEDMOREPARAMS", "461"),
            &Response::ERR_ALREADYREGISTRED => ("ERR_ALREADYREGISTRED", "462"),
            &Response::ERR_NOPERMFORHOST => ("ERR_NOPERMFORHOST", "463"),
            &Response::ERR_PASSWDMISMATCH => ("ERR_PASSWDMISMATCH", "464"),
            &Response::ERR_YOUREBANNEDCREEP => ("ERR_YOUREBANNEDCREEP", "465"),
            &Response::ERR_KEYSET => ("ERR_KEYSET", "467"),
            &Response::ERR_CHANNELISFULL => ("ERR_CHANNELISFULL", "471"),
            &Response::ERR_UNKNOWNMODE => ("ERR_UNKNOWNMODE", "472"),
            &Response::ERR_INVITEONLYCHAN => ("ERR_INVITEONLYCHAN", "473"),
            &Response::ERR_BANNEDFROMCHAN => ("ERR_BANNEDFROMCHAN", "474"),
            &Response::ERR_BADCHANNELKEY => ("ERR_BADCHANNELKEY", "475"),
            &Response::ERR_NOPRIVILEGES => ("ERR_NOPRIVILEGES", "481"),
            &Response::ERR_CHANOPRIVSNEEDED => ("ERR_CHANOPRIVSNEEDED", "482"),
            &Response::ERR_CANTKILLSERVER => ("ERR_CANTKILLSERVER", "483"),
            &Response::ERR_NOOPERHOST => ("ERR_NOOPERHOST", "491"),
            &Response::ERR_UMODEUNKNOWNFLAG => ("ERR_UMODEUNKNOWNFLAG", "501"),
            &Response::ERR_USERSDONTMATCH => ("ERR_USERSDONTMATCH", "502"),
            &Response::RPL_NONE => ("RPL_NONE", "300"),
            &Response::RPL_USERHOST => ("RPL_USERHOST", "302"),
            &Response::RPL_ISON => ("RPL_ISON", "303"),
            &Response::RPL_AWAY => ("RPL_AWAY", "301"),
            &Response::RPL_UNAWAY => ("RPL_UNAWAY", "305"),
            &Response::RPL_NOWAWAY => ("RPL_NOWAWAY", "306"),
            &Response::RPL_WHOISUSER => ("RPL_WHOISUSER", "311"),
            &Response::RPL_WHOISSERVER => ("RPL_WHOISSERVER", "312"),
            &Response::RPL_WHOISOPERATOR => ("RPL_WHOISOPERATOR", "313"),
            &Response::RPL_WHOISIDLE => ("RPL_WHOISIDLE", "317"),
            &Response::RPL_ENDOFWHOIS => ("RPL_ENDOFWHOIS", "318"),
            &Response::RPL_WHOISCHANNELS => ("RPL_WHOISCHANNELS", "319"),
            &Response::RPL_WHOWASUSER => ("RPL_WHOWASUSER", "314"),
            &Response::RPL_ENDOFWHOWAS => ("RPL_ENDOFWHOWAS", "369"),
            &Response::RPL_LISTSTART => ("RPL_LISTSTART", "321"),
            &Response::RPL_LIST => ("RPL_LIST", "322"),
            &Response::RPL_LISTEND => ("RPL_LISTEND", "323"),
            &Response::RPL_CHANNELMODEIS => ("RPL_CHANNELMODEIS", "324"),
            &Response::RPL_NOTOPIC => ("RPL_NOTOPIC", "331"),
            &Response::RPL_TOPIC => ("RPL_TOPIC", "332"),
            &Response::RPL_INVITING => ("RPL_INVITING", "341"),
            &Response::RPL_SUMMONING => ("RPL_SUMMONING", "342"),
            &Response::RPL_VERSION => ("RPL_VERSION", "351"),
            &Response::RPL_WHOREPLY => ("RPL_WHOREPLY", "352"),
            &Response::RPL_ENDOFWHO => ("RPL_ENDOFWHO", "315"),
            &Response::RPL_NAMREPLY => ("RPL_NAMREPLY", "353"),
            &Response::RPL_ENDOFNAMES => ("RPL_ENDOFNAMES", "366"),
            &Response::RPL_LINKS => ("RPL_LINKS", "364"),
            &Response::RPL_ENDOFLINKS => ("RPL_ENDOFLINKS", "365"),
            &Response::RPL_BANLIST => ("RPL_BANLIST", "367"),
            &Response::RPL_ENDOFBANLIST => ("RPL_ENDOFBANLIST", "368"),
            &Response::RPL_INFO => ("RPL_INFO", "371"),
            &Response::RPL_ENDOFINFO => ("RPL_ENDOFINFO", "374"),
            &Response::RPL_MOTDSTART => ("RPL_MOTDSTART", "375"),
            &Response::RPL_MOTD => ("RPL_MOTD", "372"),
            &Response::RPL_ENDOFMOTD => ("RPL_ENDOFMOTD", "376"),
            &Response::RPL_YOUREOPER => ("RPL_YOUREOPER", "381"),
            &Response::RPL_REHASHING => ("RPL_REHASHING", "382"),
            &Response::RPL_TIME => ("RPL_TIME", "391"),
            &Response::RPL_USERSSTART => ("RPL_USERSSTART", "392"),
            &Response::RPL_USERS => ("RPL_USERS", "393"),
            &Response::RPL_ENDOFUSERS => ("RPL_ENDOFUSERS", "394"),
            &Response::RPL_NOUSERS => ("RPL_NOUSERS", "395"),
            &Response::RPL_TRACELINK => ("RPL_TRACELINK", "200"),
            &Response::RPL_TRACECONNECTING => ("RPL_TRACECONNECTING", "201"),
            &Response::RPL_TRACEHANDSHAKE => ("RPL_TRACEHANDSHAKE", "202"),
            &Response::RPL_TRACEUNKNOWN => ("RPL_TRACEUNKNOWN", "203"),
            &Response::RPL_TRACEOPERATOR => ("RPL_TRACEOPERATOR", "204"),
            &Response::RPL_TRACEUSER => ("RPL_TRACEUSER", "205"),
            &Response::RPL_TRACESERVER => ("RPL_TRACESERVER", "206"),
            &Response::RPL_TRACENEWTYPE => ("RPL_TRACENEWTYPE", "208"),
            &Response::RPL_TRACELOG => ("RPL_TRACELOG", "261"),
            &Response::RPL_STATSLINKINFO => ("RPL_STATSLINKINFO", "211"),
            &Response::RPL_STATSCOMMANDS => ("RPL_STATSCOMMANDS", "212"),
            &Response::RPL_STATSCLINE => ("RPL_STATSCLINE", "213"),
            &Response::RPL_STATSNLINE => ("RPL_STATSNLINE", "214"),
            &Response::RPL_STATSILINE => ("RPL_STATSILINE", "215"),
            &Response::RPL_STATSKLINE => ("RPL_STATSKLINE", "216"),
            &Response::RPL_STATSYLINE => ("RPL_STATSYLINE", "218"),
            &Response::RPL_ENDOFSTATS => ("RPL_ENDOFSTATS", "219"),
            &Response::RPL_STATSLLINE => ("RPL_STATSLLINE", "241"),
            &Response::RPL_STATSUPTIME => ("RPL_STATSUPTIME", "242"),
            &Response::RPL_STATSOLINE => ("RPL_STATSOLINE", "243"),
            &Response::RPL_STATSHLINE => ("RPL_STATSHLINE", "244"),
            &Response::RPL_UMODEIS => ("RPL_UMODEIS", "221"),
            &Response::RPL_LUSERCLIENT => ("RPL_LUSERCLIENT", "251"),
            &Response::RPL_LUSEROP => ("RPL_LUSEROP", "252"),
            &Response::RPL_LUSERUNKNOWN => ("RPL_LUSERUNKNOWN", "253"),
            &Response::RPL_LUSERCHANNELS => ("RPL_LUSERCHANNELS", "254"),
            &Response::RPL_LUSERME => ("RPL_LUSERME", "255"),
            &Response::RPL_ADMINME => ("RPL_ADMINME", "256"),
            &Response::RPL_ADMINLOC1 => ("RPL_ADMINLOC1", "257"),
            &Response::RPL_ADMINLOC2 => ("RPL_ADMINLOC2", "258"),
            &Response::RPL_ADMINEMAIL => ("RPL_ADMINEMAIL", "259"),
            &Response::RPL_TRACECLASS => ("RPL_TRACECLASS", "209"),
            &Response::RPL_STATSQLINE => ("RPL_STATSQLINE", "217"),
            &Response::RPL_SERVICEINFO => ("RPL_SERVICEINFO", "231"),
            &Response::RPL_ENDOFSERVICES => ("RPL_ENDOFSERVICES", "232"),
            &Response::RPL_SERVICE => ("RPL_SERVICE", "233"),
            &Response::RPL_SERVLIST => ("RPL_SERVLIST", "234"),
            &Response::RPL_SERVLISTEND => ("RPL_SERVLISTEND", "235"),
            &Response::RPL_WHOISCHANOP => ("RPL_WHOISCHANOP", "316"),
            &Response::RPL_KILLDONE => ("RPL_KILLDONE", "361"),
            &Response::RPL_CLOSING => ("RPL_CLOSING", "362"),
            &Response::RPL_CLOSEEND => ("RPL_CLOSEEND", "363"),
            &Response::RPL_INFOSTART => ("RPL_INFOSTART", "373"),
            &Response::RPL_MYPORTIS => ("RPL_MYPORTIS", "384"),
            &Response::ERR_YOUWILLBEBANNED => ("ERR_YOUWILLBEBANNED", "466"),
            &Response::ERR_BADCHANMASK => ("ERR_BADCHANMASK", "476"),
            &Response::ERR_NOSERVICEHOST => ("ERR_NOSERVICEHOST", "492"),
            &Response::RPL_WELCOME => ("RPL_WELCOME", "001"),
            &Response::RPL_YOURHOST => ("RPL_YOURHOST", "002"),
            &Response::RPL_CREATED => ("RPL_CREATED", "003"),
            &Response::RPL_MYINFO => ("RPL_MYINFO", "004"),
            &Response::RPL_BOUNCE => ("RPL_BOUND", "005"),
        };
        write!(f, "{} {}", o.1, o.0)
    }
}

impl str::FromStr for Response {
    type Err = super::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "ERR_NOSUCHNICK" => Ok(Response::ERR_NOSUCHNICK),
            "401" => Ok(Response::ERR_NOSUCHNICK),
            "ERR_NOSUCHSERVER" => Ok(Response::ERR_NOSUCHSERVER),
            "402" => Ok(Response::ERR_NOSUCHSERVER),
            "ERR_NOSUCHCHANNEL" => Ok(Response::ERR_NOSUCHCHANNEL),
            "403" => Ok(Response::ERR_NOSUCHCHANNEL),
            "ERR_CANNOTSENDTOCHAN" => Ok(Response::ERR_CANNOTSENDTOCHAN),
            "404" => Ok(Response::ERR_CANNOTSENDTOCHAN),
            "ERR_TOOMANYCHANNELS" => Ok(Response::ERR_TOOMANYCHANNELS),
            "405" => Ok(Response::ERR_TOOMANYCHANNELS),
            "ERR_WASNOSUCHNICK" => Ok(Response::ERR_WASNOSUCHNICK),
            "406" => Ok(Response::ERR_WASNOSUCHNICK),
            "ERR_TOOMANYTARGETS" => Ok(Response::ERR_TOOMANYTARGETS),
            "407" => Ok(Response::ERR_TOOMANYTARGETS),
            "ERR_NOORIGIN" => Ok(Response::ERR_NOORIGIN),
            "409" => Ok(Response::ERR_NOORIGIN),
            "ERR_NORECIPIENT" => Ok(Response::ERR_NORECIPIENT),
            "411" => Ok(Response::ERR_NORECIPIENT),
            "ERR_NOTEXTTOSEND" => Ok(Response::ERR_NOTEXTTOSEND),
            "412" => Ok(Response::ERR_NOTEXTTOSEND),
            "ERR_NOTOPLEVEL" => Ok(Response::ERR_NOTOPLEVEL),
            "413" => Ok(Response::ERR_NOTOPLEVEL),
            "ERR_WILDTOPLEVEL" => Ok(Response::ERR_WILDTOPLEVEL),
            "414" => Ok(Response::ERR_WILDTOPLEVEL),
            "ERR_UNKNOWNCOMMAND" => Ok(Response::ERR_UNKNOWNCOMMAND),
            "421" => Ok(Response::ERR_UNKNOWNCOMMAND),
            "ERR_NOMOTD" => Ok(Response::ERR_NOMOTD),
            "422" => Ok(Response::ERR_NOMOTD),
            "ERR_NOADMININFO" => Ok(Response::ERR_NOADMININFO),
            "423" => Ok(Response::ERR_NOADMININFO),
            "ERR_FILEERROR" => Ok(Response::ERR_FILEERROR),
            "424" => Ok(Response::ERR_FILEERROR),
            "ERR_NONICKNAMEGIVEN" => Ok(Response::ERR_NONICKNAMEGIVEN),
            "431" => Ok(Response::ERR_NONICKNAMEGIVEN),
            "ERR_ERRONEUSNICKNAME" => Ok(Response::ERR_ERRONEUSNICKNAME),
            "432" => Ok(Response::ERR_ERRONEUSNICKNAME),
            "ERR_NICKNAMEINUSE" => Ok(Response::ERR_NICKNAMEINUSE),
            "433" => Ok(Response::ERR_NICKNAMEINUSE),
            "ERR_NICKCOLLISION" => Ok(Response::ERR_NICKCOLLISION),
            "436" => Ok(Response::ERR_NICKCOLLISION),
            "ERR_USERNOTINCHANNEL" => Ok(Response::ERR_USERNOTINCHANNEL),
            "441" => Ok(Response::ERR_USERNOTINCHANNEL),
            "ERR_NOTONCHANNEL" => Ok(Response::ERR_NOTONCHANNEL),
            "442" => Ok(Response::ERR_NOTONCHANNEL),
            "ERR_USERONCHANNEL" => Ok(Response::ERR_USERONCHANNEL),
            "443" => Ok(Response::ERR_USERONCHANNEL),
            "ERR_NOLOGIN" => Ok(Response::ERR_NOLOGIN),
            "444" => Ok(Response::ERR_NOLOGIN),
            "ERR_SUMMONDISABLED" => Ok(Response::ERR_SUMMONDISABLED),
            "445" => Ok(Response::ERR_SUMMONDISABLED),
            "ERR_USERSDISABLED" => Ok(Response::ERR_USERSDISABLED),
            "446" => Ok(Response::ERR_USERSDISABLED),
            "ERR_NOTREGISTERED" => Ok(Response::ERR_NOTREGISTERED),
            "451" => Ok(Response::ERR_NOTREGISTERED),
            "ERR_NEEDMOREPARAMS" => Ok(Response::ERR_NEEDMOREPARAMS),
            "461" => Ok(Response::ERR_NEEDMOREPARAMS),
            "ERR_ALREADYREGISTRED" => Ok(Response::ERR_ALREADYREGISTRED),
            "462" => Ok(Response::ERR_ALREADYREGISTRED),
            "ERR_NOPERMFORHOST" => Ok(Response::ERR_NOPERMFORHOST),
            "463" => Ok(Response::ERR_NOPERMFORHOST),
            "ERR_PASSWDMISMATCH" => Ok(Response::ERR_PASSWDMISMATCH),
            "464" => Ok(Response::ERR_PASSWDMISMATCH),
            "ERR_YOUREBANNEDCREEP" => Ok(Response::ERR_YOUREBANNEDCREEP),
            "465" => Ok(Response::ERR_YOUREBANNEDCREEP),
            "ERR_KEYSET" => Ok(Response::ERR_KEYSET),
            "467" => Ok(Response::ERR_KEYSET),
            "ERR_CHANNELISFULL" => Ok(Response::ERR_CHANNELISFULL),
            "471" => Ok(Response::ERR_CHANNELISFULL),
            "ERR_UNKNOWNMODE" => Ok(Response::ERR_UNKNOWNMODE),
            "472" => Ok(Response::ERR_UNKNOWNMODE),
            "ERR_INVITEONLYCHAN" => Ok(Response::ERR_INVITEONLYCHAN),
            "473" => Ok(Response::ERR_INVITEONLYCHAN),
            "ERR_BANNEDFROMCHAN" => Ok(Response::ERR_BANNEDFROMCHAN),
            "474" => Ok(Response::ERR_BANNEDFROMCHAN),
            "ERR_BADCHANNELKEY" => Ok(Response::ERR_BADCHANNELKEY),
            "475" => Ok(Response::ERR_BADCHANNELKEY),
            "ERR_NOPRIVILEGES" => Ok(Response::ERR_NOPRIVILEGES),
            "481" => Ok(Response::ERR_NOPRIVILEGES),
            "ERR_CHANOPRIVSNEEDED" => Ok(Response::ERR_CHANOPRIVSNEEDED),
            "482" => Ok(Response::ERR_CHANOPRIVSNEEDED),
            "ERR_CANTKILLSERVER" => Ok(Response::ERR_CANTKILLSERVER),
            "483" => Ok(Response::ERR_CANTKILLSERVER),
            "ERR_NOOPERHOST" => Ok(Response::ERR_NOOPERHOST),
            "491" => Ok(Response::ERR_NOOPERHOST),
            "ERR_UMODEUNKNOWNFLAG" => Ok(Response::ERR_UMODEUNKNOWNFLAG),
            "501" => Ok(Response::ERR_UMODEUNKNOWNFLAG),
            "ERR_USERSDONTMATCH" => Ok(Response::ERR_USERSDONTMATCH),
            "502" => Ok(Response::ERR_USERSDONTMATCH),
            "RPL_NONE" => Ok(Response::RPL_NONE),
            "300" => Ok(Response::RPL_NONE),
            "RPL_USERHOST" => Ok(Response::RPL_USERHOST),
            "302" => Ok(Response::RPL_USERHOST),
            "RPL_ISON" => Ok(Response::RPL_ISON),
            "303" => Ok(Response::RPL_ISON),
            "RPL_AWAY" => Ok(Response::RPL_AWAY),
            "301" => Ok(Response::RPL_AWAY),
            "RPL_UNAWAY" => Ok(Response::RPL_UNAWAY),
            "305" => Ok(Response::RPL_UNAWAY),
            "RPL_NOWAWAY" => Ok(Response::RPL_NOWAWAY),
            "306" => Ok(Response::RPL_NOWAWAY),
            "RPL_WHOISUSER" => Ok(Response::RPL_WHOISUSER),
            "311" => Ok(Response::RPL_WHOISUSER),
            "RPL_WHOISSERVER" => Ok(Response::RPL_WHOISSERVER),
            "312" => Ok(Response::RPL_WHOISSERVER),
            "RPL_WHOISOPERATOR" => Ok(Response::RPL_WHOISOPERATOR),
            "313" => Ok(Response::RPL_WHOISOPERATOR),
            "RPL_WHOISIDLE" => Ok(Response::RPL_WHOISIDLE),
            "317" => Ok(Response::RPL_WHOISIDLE),
            "RPL_ENDOFWHOIS" => Ok(Response::RPL_ENDOFWHOIS),
            "318" => Ok(Response::RPL_ENDOFWHOIS),
            "RPL_WHOISCHANNELS" => Ok(Response::RPL_WHOISCHANNELS),
            "319" => Ok(Response::RPL_WHOISCHANNELS),
            "RPL_WHOWASUSER" => Ok(Response::RPL_WHOWASUSER),
            "314" => Ok(Response::RPL_WHOWASUSER),
            "RPL_ENDOFWHOWAS" => Ok(Response::RPL_ENDOFWHOWAS),
            "369" => Ok(Response::RPL_ENDOFWHOWAS),
            "RPL_LISTSTART" => Ok(Response::RPL_LISTSTART),
            "321" => Ok(Response::RPL_LISTSTART),
            "RPL_LIST" => Ok(Response::RPL_LIST),
            "322" => Ok(Response::RPL_LIST),
            "RPL_LISTEND" => Ok(Response::RPL_LISTEND),
            "323" => Ok(Response::RPL_LISTEND),
            "RPL_CHANNELMODEIS" => Ok(Response::RPL_CHANNELMODEIS),
            "324" => Ok(Response::RPL_CHANNELMODEIS),
            "RPL_NOTOPIC" => Ok(Response::RPL_NOTOPIC),
            "331" => Ok(Response::RPL_NOTOPIC),
            "RPL_TOPIC" => Ok(Response::RPL_TOPIC),
            "332" => Ok(Response::RPL_TOPIC),
            "RPL_INVITING" => Ok(Response::RPL_INVITING),
            "341" => Ok(Response::RPL_INVITING),
            "RPL_SUMMONING" => Ok(Response::RPL_SUMMONING),
            "342" => Ok(Response::RPL_SUMMONING),
            "RPL_VERSION" => Ok(Response::RPL_VERSION),
            "351" => Ok(Response::RPL_VERSION),
            "RPL_WHOREPLY" => Ok(Response::RPL_WHOREPLY),
            "352" => Ok(Response::RPL_WHOREPLY),
            "RPL_ENDOFWHO" => Ok(Response::RPL_ENDOFWHO),
            "315" => Ok(Response::RPL_ENDOFWHO),
            "RPL_NAMREPLY" => Ok(Response::RPL_NAMREPLY),
            "353" => Ok(Response::RPL_NAMREPLY),
            "RPL_ENDOFNAMES" => Ok(Response::RPL_ENDOFNAMES),
            "366" => Ok(Response::RPL_ENDOFNAMES),
            "RPL_LINKS" => Ok(Response::RPL_LINKS),
            "364" => Ok(Response::RPL_LINKS),
            "RPL_ENDOFLINKS" => Ok(Response::RPL_ENDOFLINKS),
            "365" => Ok(Response::RPL_ENDOFLINKS),
            "RPL_BANLIST" => Ok(Response::RPL_BANLIST),
            "367" => Ok(Response::RPL_BANLIST),
            "RPL_ENDOFBANLIST" => Ok(Response::RPL_ENDOFBANLIST),
            "368" => Ok(Response::RPL_ENDOFBANLIST),
            "RPL_INFO" => Ok(Response::RPL_INFO),
            "371" => Ok(Response::RPL_INFO),
            "RPL_ENDOFINFO" => Ok(Response::RPL_ENDOFINFO),
            "374" => Ok(Response::RPL_ENDOFINFO),
            "RPL_MOTDSTART" => Ok(Response::RPL_MOTDSTART),
            "375" => Ok(Response::RPL_MOTDSTART),
            "RPL_MOTD" => Ok(Response::RPL_MOTD),
            "372" => Ok(Response::RPL_MOTD),
            "RPL_ENDOFMOTD" => Ok(Response::RPL_ENDOFMOTD),
            "376" => Ok(Response::RPL_ENDOFMOTD),
            "RPL_YOUREOPER" => Ok(Response::RPL_YOUREOPER),
            "381" => Ok(Response::RPL_YOUREOPER),
            "RPL_REHASHING" => Ok(Response::RPL_REHASHING),
            "382" => Ok(Response::RPL_REHASHING),
            "RPL_TIME" => Ok(Response::RPL_TIME),
            "391" => Ok(Response::RPL_TIME),
            "RPL_USERSSTART" => Ok(Response::RPL_USERSSTART),
            "392" => Ok(Response::RPL_USERSSTART),
            "RPL_USERS" => Ok(Response::RPL_USERS),
            "393" => Ok(Response::RPL_USERS),
            "RPL_ENDOFUSERS" => Ok(Response::RPL_ENDOFUSERS),
            "394" => Ok(Response::RPL_ENDOFUSERS),
            "RPL_NOUSERS" => Ok(Response::RPL_NOUSERS),
            "395" => Ok(Response::RPL_NOUSERS),
            "RPL_TRACELINK" => Ok(Response::RPL_TRACELINK),
            "200" => Ok(Response::RPL_TRACELINK),
            "RPL_TRACECONNECTING" => Ok(Response::RPL_TRACECONNECTING),
            "201" => Ok(Response::RPL_TRACECONNECTING),
            "RPL_TRACEHANDSHAKE" => Ok(Response::RPL_TRACEHANDSHAKE),
            "202" => Ok(Response::RPL_TRACEHANDSHAKE),
            "RPL_TRACEUNKNOWN" => Ok(Response::RPL_TRACEUNKNOWN),
            "203" => Ok(Response::RPL_TRACEUNKNOWN),
            "RPL_TRACEOPERATOR" => Ok(Response::RPL_TRACEOPERATOR),
            "204" => Ok(Response::RPL_TRACEOPERATOR),
            "RPL_TRACEUSER" => Ok(Response::RPL_TRACEUSER),
            "205" => Ok(Response::RPL_TRACEUSER),
            "RPL_TRACESERVER" => Ok(Response::RPL_TRACESERVER),
            "206" => Ok(Response::RPL_TRACESERVER),
            "RPL_TRACENEWTYPE" => Ok(Response::RPL_TRACENEWTYPE),
            "208" => Ok(Response::RPL_TRACENEWTYPE),
            "RPL_TRACELOG" => Ok(Response::RPL_TRACELOG),
            "261" => Ok(Response::RPL_TRACELOG),
            "RPL_STATSLINKINFO" => Ok(Response::RPL_STATSLINKINFO),
            "211" => Ok(Response::RPL_STATSLINKINFO),
            "RPL_STATSCOMMANDS" => Ok(Response::RPL_STATSCOMMANDS),
            "212" => Ok(Response::RPL_STATSCOMMANDS),
            "RPL_STATSCLINE" => Ok(Response::RPL_STATSCLINE),
            "213" => Ok(Response::RPL_STATSCLINE),
            "RPL_STATSNLINE" => Ok(Response::RPL_STATSNLINE),
            "214" => Ok(Response::RPL_STATSNLINE),
            "RPL_STATSILINE" => Ok(Response::RPL_STATSILINE),
            "215" => Ok(Response::RPL_STATSILINE),
            "RPL_STATSKLINE" => Ok(Response::RPL_STATSKLINE),
            "216" => Ok(Response::RPL_STATSKLINE),
            "RPL_STATSYLINE" => Ok(Response::RPL_STATSYLINE),
            "218" => Ok(Response::RPL_STATSYLINE),
            "RPL_ENDOFSTATS" => Ok(Response::RPL_ENDOFSTATS),
            "219" => Ok(Response::RPL_ENDOFSTATS),
            "RPL_STATSLLINE" => Ok(Response::RPL_STATSLLINE),
            "241" => Ok(Response::RPL_STATSLLINE),
            "RPL_STATSUPTIME" => Ok(Response::RPL_STATSUPTIME),
            "242" => Ok(Response::RPL_STATSUPTIME),
            "RPL_STATSOLINE" => Ok(Response::RPL_STATSOLINE),
            "243" => Ok(Response::RPL_STATSOLINE),
            "RPL_STATSHLINE" => Ok(Response::RPL_STATSHLINE),
            "244" => Ok(Response::RPL_STATSHLINE),
            "RPL_UMODEIS" => Ok(Response::RPL_UMODEIS),
            "221" => Ok(Response::RPL_UMODEIS),
            "RPL_LUSERCLIENT" => Ok(Response::RPL_LUSERCLIENT),
            "251" => Ok(Response::RPL_LUSERCLIENT),
            "RPL_LUSEROP" => Ok(Response::RPL_LUSEROP),
            "252" => Ok(Response::RPL_LUSEROP),
            "RPL_LUSERUNKNOWN" => Ok(Response::RPL_LUSERUNKNOWN),
            "253" => Ok(Response::RPL_LUSERUNKNOWN),
            "RPL_LUSERCHANNELS" => Ok(Response::RPL_LUSERCHANNELS),
            "254" => Ok(Response::RPL_LUSERCHANNELS),
            "RPL_LUSERME" => Ok(Response::RPL_LUSERME),
            "255" => Ok(Response::RPL_LUSERME),
            "RPL_ADMINME" => Ok(Response::RPL_ADMINME),
            "256" => Ok(Response::RPL_ADMINME),
            "RPL_ADMINLOC1" => Ok(Response::RPL_ADMINLOC1),
            "257" => Ok(Response::RPL_ADMINLOC1),
            "RPL_ADMINLOC2" => Ok(Response::RPL_ADMINLOC2),
            "258" => Ok(Response::RPL_ADMINLOC2),
            "RPL_ADMINEMAIL" => Ok(Response::RPL_ADMINEMAIL),
            "259" => Ok(Response::RPL_ADMINEMAIL),
            "RPL_TRACECLASS" => Ok(Response::RPL_TRACECLASS),
            "209" => Ok(Response::RPL_TRACECLASS),
            "RPL_STATSQLINE" => Ok(Response::RPL_STATSQLINE),
            "217" => Ok(Response::RPL_STATSQLINE),
            "RPL_SERVICEINFO" => Ok(Response::RPL_SERVICEINFO),
            "231" => Ok(Response::RPL_SERVICEINFO),
            "RPL_ENDOFSERVICES" => Ok(Response::RPL_ENDOFSERVICES),
            "232" => Ok(Response::RPL_ENDOFSERVICES),
            "RPL_SERVICE" => Ok(Response::RPL_SERVICE),
            "233" => Ok(Response::RPL_SERVICE),
            "RPL_SERVLIST" => Ok(Response::RPL_SERVLIST),
            "234" => Ok(Response::RPL_SERVLIST),
            "RPL_SERVLISTEND" => Ok(Response::RPL_SERVLISTEND),
            "235" => Ok(Response::RPL_SERVLISTEND),
            "RPL_WHOISCHANOP" => Ok(Response::RPL_WHOISCHANOP),
            "316" => Ok(Response::RPL_WHOISCHANOP),
            "RPL_KILLDONE" => Ok(Response::RPL_KILLDONE),
            "361" => Ok(Response::RPL_KILLDONE),
            "RPL_CLOSING" => Ok(Response::RPL_CLOSING),
            "362" => Ok(Response::RPL_CLOSING),
            "RPL_CLOSEEND" => Ok(Response::RPL_CLOSEEND),
            "363" => Ok(Response::RPL_CLOSEEND),
            "RPL_INFOSTART" => Ok(Response::RPL_INFOSTART),
            "373" => Ok(Response::RPL_INFOSTART),
            "RPL_MYPORTIS" => Ok(Response::RPL_MYPORTIS),
            "384" => Ok(Response::RPL_MYPORTIS),
            "ERR_YOUWILLBEBANNED" => Ok(Response::ERR_YOUWILLBEBANNED),
            "466" => Ok(Response::ERR_YOUWILLBEBANNED),
            "ERR_BADCHANMASK" => Ok(Response::ERR_BADCHANMASK),
            "476" => Ok(Response::ERR_BADCHANMASK),
            "ERR_NOSERVICEHOST" => Ok(Response::ERR_NOSERVICEHOST),
            "492" => Ok(Response::ERR_NOSERVICEHOST),
            "RPL_WELCOME" => Ok(Response::RPL_WELCOME),
            "001" => Ok(Response::RPL_WELCOME),
            "RPL_YOURHOST" => Ok(Response::RPL_YOURHOST),
            "002" => Ok(Response::RPL_YOURHOST),
            "RPL_CREATED" => Ok(Response::RPL_CREATED),
            "003" => Ok(Response::RPL_CREATED),
            "RPL_MYINFO" => Ok(Response::RPL_MYINFO),
            "004" => Ok(Response::RPL_MYINFO),
            "RPL_BOUNCE" => Ok(Response::RPL_BOUNCE),
            "005" => Ok(Response::RPL_BOUNCE),
            _ => unimplemented!(), //Err(super::ParseError::new("cannot parse reply string")),
        }
    }
}
