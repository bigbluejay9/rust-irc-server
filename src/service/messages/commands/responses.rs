use std::fmt::{self, Formatter, Error as FmtError};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOSUCHNICK {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOSUCHSERVER {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOSUCHCHANNEL {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CANNOTSENDTOCHAN {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TOOMANYCHANNELS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WASNOSUCHNICK {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TOOMANYTARGETS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOORIGIN {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NORECIPIENT {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOTEXTTOSEND {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOTOPLEVEL {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WILDTOPLEVEL {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UNKNOWNCOMMAND {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOMOTD {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOADMININFO {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct FILEERROR {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NONICKNAMEGIVEN {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ERRONEUSNICKNAME {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NICKNAMEINUSE {
    pub nick: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NICKCOLLISION {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct USERNOTINCHANNEL {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOTONCHANNEL {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct USERONCHANNEL {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOLOGIN {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SUMMONDISABLED {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct USERSDISABLED {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOTREGISTERED {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NeedMoreParams {
    pub command: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct AlreadyRegistered {
    pub nick: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOPERMFORHOST {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct PASSWDMISMATCH {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct YOUREBANNEDCREEP {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct KEYSET {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CHANNELISFULL {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UNKNOWNMODE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct INVITEONLYCHAN {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BANNEDFROMCHAN {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BADCHANNELKEY {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOPRIVILEGES {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CHANOPRIVSNEEDED {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CANTKILLSERVER {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOOPERHOST {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UModeUnknownFlag {
    pub nick: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UsersDontMatch {
    pub nick: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NONE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct USERHOST {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ISON {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct AWAY {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UNAWAY {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOWAWAY {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WHOISUSER {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WHOISSERVER {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WHOISOPERATOR {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WHOISIDLE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFWHOIS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WHOISCHANNELS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WHOWASUSER {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFWHOWAS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LISTSTART {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LIST {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LISTEND {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CHANNELMODEIS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOTOPIC {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Topic {
    pub nick: String,
    pub channel: String,
    pub topic: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct INVITING {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SUMMONING {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct VERSION {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WHOREPLY {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFWHO {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NamReply {
    pub nick: String,
    pub symbol: String,
    pub channel: String,
    // (Prefix, Nick).
    pub members: Vec<(String, String)>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFNAMES {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LINKS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFLINKS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BANLIST {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFBANLIST {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct INFO {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFINFO {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct MOTDSTART {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct MOTD {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFMOTD {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct YOUREOPER {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct REHASHING {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TIME {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct USERSSTART {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct USERS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFUSERS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOUSERS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACELINK {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACECONNECTING {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACEHANDSHAKE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACEUNKNOWN {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACEOPERATOR {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACEUSER {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACESERVER {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACENEWTYPE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACELOG {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSLINKINFO {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSCOMMANDS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSCLINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSNLINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSILINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSKLINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSYLINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFSTATS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSLLINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSUPTIME {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSOLINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSHLINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UMODEIS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LUSERCLIENT {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LUSEROP {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LUSERUNKNOWN {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LUSERCHANNELS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LUSERME {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ADMINME {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ADMINLOC1 {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ADMINLOC2 {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ADMINEMAIL {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TRACECLASS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct STATSQLINE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SERVICEINFO {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ENDOFSERVICES {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SERVICE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SERVLIST {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SERVLISTEND {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct WHOISCHANOP {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct KILLDONE {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CLOSING {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CLOSEEND {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct INFOSTART {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct MYPORTIS {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct YOUWILLBEBANNED {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BADCHANMASK {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NOSERVICEHOST {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Welcome {
    pub nick: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct YourHost {
    pub nick: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Created {
    pub nick: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct MyInfo {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ISUPPORT {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BOUNCE {}

impl fmt::Display for NOSUCHNICK {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "401")
    }
}

impl fmt::Display for NOSUCHSERVER {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "402")
    }
}

impl fmt::Display for NOSUCHCHANNEL {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "403")
    }
}

impl fmt::Display for CANNOTSENDTOCHAN {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "404")
    }
}

impl fmt::Display for TOOMANYCHANNELS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "405")
    }
}

impl fmt::Display for WASNOSUCHNICK {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "406")
    }
}

impl fmt::Display for TOOMANYTARGETS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "407")
    }
}

impl fmt::Display for NOORIGIN {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "409")
    }
}

impl fmt::Display for NORECIPIENT {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "411")
    }
}

impl fmt::Display for NOTEXTTOSEND {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "412")
    }
}

impl fmt::Display for NOTOPLEVEL {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "413")
    }
}

impl fmt::Display for WILDTOPLEVEL {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "414")
    }
}

impl fmt::Display for UNKNOWNCOMMAND {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "421")
    }
}

impl fmt::Display for NOMOTD {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "422")
    }
}

impl fmt::Display for NOADMININFO {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "423")
    }
}

impl fmt::Display for FILEERROR {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "424")
    }
}

impl fmt::Display for NONICKNAMEGIVEN {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "431")
    }
}

impl fmt::Display for ERRONEUSNICKNAME {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "432")
    }
}

impl fmt::Display for NICKNAMEINUSE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "433 {} :Nickname is already in use", self.nick)
    }
}

impl fmt::Display for NICKCOLLISION {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "436")
    }
}

impl fmt::Display for USERNOTINCHANNEL {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "441")
    }
}

impl fmt::Display for NOTONCHANNEL {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "442")
    }
}

impl fmt::Display for USERONCHANNEL {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "443")
    }
}

impl fmt::Display for NOLOGIN {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "444")
    }
}

impl fmt::Display for SUMMONDISABLED {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "445")
    }
}

impl fmt::Display for USERSDISABLED {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "446")
    }
}

impl fmt::Display for NOTREGISTERED {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "451 :You have not registered")
    }
}

impl fmt::Display for NeedMoreParams {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "461 {} :Not enough parameters", self.command)
    }
}

impl fmt::Display for AlreadyRegistered {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "462 {} :You may not reregister", self.nick)
    }
}

impl fmt::Display for NOPERMFORHOST {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "463")
    }
}

impl fmt::Display for PASSWDMISMATCH {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "464")
    }
}

impl fmt::Display for YOUREBANNEDCREEP {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "465")
    }
}

impl fmt::Display for KEYSET {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "467")
    }
}

impl fmt::Display for CHANNELISFULL {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "471")
    }
}

impl fmt::Display for UNKNOWNMODE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "472")
    }
}

impl fmt::Display for INVITEONLYCHAN {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "473")
    }
}

impl fmt::Display for BANNEDFROMCHAN {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "474")
    }
}

impl fmt::Display for BADCHANNELKEY {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "475")
    }
}

impl fmt::Display for NOPRIVILEGES {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "481")
    }
}

impl fmt::Display for CHANOPRIVSNEEDED {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "482")
    }
}

impl fmt::Display for CANTKILLSERVER {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "483")
    }
}

impl fmt::Display for NOOPERHOST {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "491")
    }
}

impl fmt::Display for UModeUnknownFlag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "501 {} :Unknown MODE flag", self.nick)
    }
}

impl fmt::Display for UsersDontMatch {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "502 {} :Cant change mode for other users", self.nick)
    }
}

impl fmt::Display for NONE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "300")
    }
}

impl fmt::Display for USERHOST {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "302")
    }
}

impl fmt::Display for ISON {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "303")
    }
}

impl fmt::Display for AWAY {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "301")
    }
}

impl fmt::Display for UNAWAY {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "305")
    }
}

impl fmt::Display for NOWAWAY {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "306")
    }
}

impl fmt::Display for WHOISUSER {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "311")
    }
}

impl fmt::Display for WHOISSERVER {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "312")
    }
}

impl fmt::Display for WHOISOPERATOR {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "313")
    }
}

impl fmt::Display for WHOISIDLE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "317")
    }
}

impl fmt::Display for ENDOFWHOIS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "318")
    }
}

impl fmt::Display for WHOISCHANNELS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "319")
    }
}

impl fmt::Display for WHOWASUSER {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "314")
    }
}

impl fmt::Display for ENDOFWHOWAS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "369")
    }
}

impl fmt::Display for LISTSTART {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "321")
    }
}

impl fmt::Display for LIST {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "322")
    }
}

impl fmt::Display for LISTEND {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "323")
    }
}

impl fmt::Display for CHANNELMODEIS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "324")
    }
}

impl fmt::Display for NOTOPIC {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "331")
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "332 {} {} :{}", self.nick, self.channel, self.topic)
    }
}

impl fmt::Display for INVITING {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "341")
    }
}

impl fmt::Display for SUMMONING {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "342")
    }
}

impl fmt::Display for VERSION {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "351")
    }
}

impl fmt::Display for WHOREPLY {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "352")
    }
}

impl fmt::Display for ENDOFWHO {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "315")
    }
}

impl fmt::Display for NamReply {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "353 {} {} {} :", self.nick, self.symbol, self.channel)?;
        for &(ref pre, ref n) in self.members.iter() {
            write!(f, "{}{}", pre, n)?;
        }
        Ok(())
    }
}

impl fmt::Display for ENDOFNAMES {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "366")
    }
}

impl fmt::Display for LINKS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "364")
    }
}

impl fmt::Display for ENDOFLINKS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "365")
    }
}

impl fmt::Display for BANLIST {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "367")
    }
}

impl fmt::Display for ENDOFBANLIST {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "368")
    }
}

impl fmt::Display for INFO {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "371")
    }
}

impl fmt::Display for ENDOFINFO {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "374")
    }
}

impl fmt::Display for MOTDSTART {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "375")
    }
}

impl fmt::Display for MOTD {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "372")
    }
}

impl fmt::Display for ENDOFMOTD {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "376")
    }
}

impl fmt::Display for YOUREOPER {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "381")
    }
}

impl fmt::Display for REHASHING {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "382")
    }
}

impl fmt::Display for TIME {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "391")
    }
}

impl fmt::Display for USERSSTART {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "392")
    }
}

impl fmt::Display for USERS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "393")
    }
}

impl fmt::Display for ENDOFUSERS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "394")
    }
}

impl fmt::Display for NOUSERS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "395")
    }
}

impl fmt::Display for TRACELINK {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "200")
    }
}

impl fmt::Display for TRACECONNECTING {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "201")
    }
}

impl fmt::Display for TRACEHANDSHAKE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "202")
    }
}

impl fmt::Display for TRACEUNKNOWN {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "203")
    }
}

impl fmt::Display for TRACEOPERATOR {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "204")
    }
}

impl fmt::Display for TRACEUSER {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "205")
    }
}

impl fmt::Display for TRACESERVER {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "206")
    }
}

impl fmt::Display for TRACENEWTYPE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "208")
    }
}

impl fmt::Display for TRACELOG {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "261")
    }
}

impl fmt::Display for STATSLINKINFO {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "211")
    }
}

impl fmt::Display for STATSCOMMANDS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "212")
    }
}

impl fmt::Display for STATSCLINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "213")
    }
}

impl fmt::Display for STATSNLINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "214")
    }
}

impl fmt::Display for STATSILINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "215")
    }
}

impl fmt::Display for STATSKLINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "216")
    }
}

impl fmt::Display for STATSYLINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "218")
    }
}

impl fmt::Display for ENDOFSTATS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "219")
    }
}

impl fmt::Display for STATSLLINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "241")
    }
}

impl fmt::Display for STATSUPTIME {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "242")
    }
}

impl fmt::Display for STATSOLINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "243")
    }
}

impl fmt::Display for STATSHLINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "244")
    }
}

impl fmt::Display for UMODEIS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "221")
    }
}

impl fmt::Display for LUSERCLIENT {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "251")
    }
}

impl fmt::Display for LUSEROP {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "252")
    }
}

impl fmt::Display for LUSERUNKNOWN {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "253")
    }
}

impl fmt::Display for LUSERCHANNELS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "254")
    }
}

impl fmt::Display for LUSERME {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "255")
    }
}

impl fmt::Display for ADMINME {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "256")
    }
}

impl fmt::Display for ADMINLOC1 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "257")
    }
}

impl fmt::Display for ADMINLOC2 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "258")
    }
}

impl fmt::Display for ADMINEMAIL {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "259")
    }
}

impl fmt::Display for TRACECLASS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "209")
    }
}

impl fmt::Display for STATSQLINE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "217")
    }
}

impl fmt::Display for SERVICEINFO {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "231")
    }
}

impl fmt::Display for ENDOFSERVICES {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "232")
    }
}

impl fmt::Display for SERVICE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "233")
    }
}

impl fmt::Display for SERVLIST {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "234")
    }
}

impl fmt::Display for SERVLISTEND {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "235")
    }
}

impl fmt::Display for WHOISCHANOP {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "316")
    }
}

impl fmt::Display for KILLDONE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "361")
    }
}

impl fmt::Display for CLOSING {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "362")
    }
}

impl fmt::Display for CLOSEEND {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "363")
    }
}

impl fmt::Display for INFOSTART {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "373")
    }
}

impl fmt::Display for MYPORTIS {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "384")
    }
}

impl fmt::Display for YOUWILLBEBANNED {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "466")
    }
}

impl fmt::Display for BADCHANMASK {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "476")
    }
}

impl fmt::Display for NOSERVICEHOST {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "492")
    }
}

impl fmt::Display for Welcome {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "001 {} :{}", self.nick, self.message)
    }
}

impl fmt::Display for YourHost {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "002 {} :{}", self.nick, self.message)
    }
}

impl fmt::Display for Created {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "003 {} :{}", self.nick, self.message)
    }
}

impl fmt::Display for MyInfo {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "004")
    }
}

impl fmt::Display for ISUPPORT {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "005")
    }
}

impl fmt::Display for BOUNCE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "010")
    }
}
