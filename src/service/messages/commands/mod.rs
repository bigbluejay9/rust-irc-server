pub mod requests;
pub mod responses;

use std::{str, fmt};
use std::fmt::{Formatter, Error as FmtError};
use super::{next_token, ParseError};

// RFC 1459 4, 5. RFC 2812.
#[allow(non_snake_case, non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    // 4.1 Connection Registration.
    NICK(requests::Nick),
    PASS(requests::Pass),
    USER(requests::User),
    SERVER(requests::Server),
    OPER(requests::Oper),
    SERVICE(requests::Service),
    QUIT(requests::Quit),
    SQUIT(requests::Squit),
    // 4.2 Channel Operations.
    JOIN(requests::Join),
    PART(requests::Part),
    // TODO(lazau): Verify.
    MODE(requests::Mode),
    TOPIC(requests::Topic),
    NAMES(requests::Names),
    LIST(requests::List),
    INVITE(requests::Invite),
    KICK(requests::Kick),
    // 4.3 Server queries and commands.
    // RFC 2812 additions.
    MOTD(requests::Motd),
    LUSERS(requests::Lusers),
    // END RFC 2812 additions.
    VERSION(requests::Version),
    STATS(requests::Stats),
    // TODO(lazau): Server mask should be a type.
    LINKS(requests::Links),
    TIME(requests::Time),
    CONNECT(requests::Connect),
    TRACE(requests::Trace),
    ADMIN(requests::Admin),
    INFO(requests::Info),
    // 4.4 Sending messages.
    PRIVMSG(requests::Privmsg),
    NOTICE(requests::Notice),
    // 4.5 User based queries.
    // RFC 2812 additions (Service Query and Commands).
    SERVLIST(requests::Servlist),
    SQUERY(requests::Squery),
    // END RFC 2812 additions.
    WHO(requests::Who),
    WHOIS(requests::Whois),
    WHOWAS(requests::Whowas),
    // 4.6 Misc.
    KILL(requests::Kill),
    PING(requests::Ping),
    PONG(requests::Pong),
    ERROR(requests::Error),

    // 5 Optionals.
    AWAY(requests::Away),
    REHASH(requests::Rehash),
    RESTART(requests::Restart),
    SUMMON(requests::Summon),
    USERS(requests::Users),
    WALLOPS(requests::Wallops),
    USERHOST(requests::Userhost),
    ISON(requests::Ison),

    // RFC 1459 6
    // 6.1 Error replies.
    ERR_NOSUCHNICK(responses::NOSUCHNICK),
    ERR_NOSUCHSERVER(responses::NOSUCHSERVER),
    ERR_NOSUCHCHANNEL(responses::NoSuchChannel),
    ERR_CANNOTSENDTOCHAN(responses::CANNOTSENDTOCHAN),
    ERR_TOOMANYCHANNELS(responses::TOOMANYCHANNELS),
    ERR_WASNOSUCHNICK(responses::WASNOSUCHNICK),
    ERR_TOOMANYTARGETS(responses::TOOMANYTARGETS),
    ERR_NOORIGIN(responses::NOORIGIN),
    ERR_NORECIPIENT(responses::NORECIPIENT),
    ERR_NOTEXTTOSEND(responses::NOTEXTTOSEND),
    ERR_NOTOPLEVEL(responses::NOTOPLEVEL),
    ERR_WILDTOPLEVEL(responses::WILDTOPLEVEL),
    ERR_UNKNOWNCOMMAND(responses::UNKNOWNCOMMAND),
    ERR_NOMOTD(responses::NOMOTD),
    ERR_NOADMININFO(responses::NOADMININFO),
    ERR_FILEERROR(responses::FILEERROR),
    ERR_NONICKNAMEGIVEN(responses::NONICKNAMEGIVEN),
    ERR_ERRONEUSNICKNAME(responses::ERRONEUSNICKNAME),
    ERR_NICKNAMEINUSE(responses::NICKNAMEINUSE),
    ERR_NICKCOLLISION(responses::NICKCOLLISION),
    ERR_USERNOTINCHANNEL(responses::USERNOTINCHANNEL),
    ERR_NOTONCHANNEL(responses::NotOnChannel),
    ERR_USERONCHANNEL(responses::USERONCHANNEL),
    ERR_NOLOGIN(responses::NOLOGIN),
    ERR_SUMMONDISABLED(responses::SUMMONDISABLED),
    ERR_USERSDISABLED(responses::USERSDISABLED),
    ERR_NOTREGISTERED(responses::NOTREGISTERED),
    ERR_NEEDMOREPARAMS(responses::NeedMoreParams),
    ERR_ALREADYREGISTRED(responses::AlreadyRegistered),
    ERR_NOPERMFORHOST(responses::NOPERMFORHOST),
    ERR_PASSWDMISMATCH(responses::PASSWDMISMATCH),
    ERR_YOUREBANNEDCREEP(responses::YOUREBANNEDCREEP),
    ERR_KEYSET(responses::KEYSET),
    ERR_CHANNELISFULL(responses::CHANNELISFULL),
    ERR_UNKNOWNMODE(responses::UNKNOWNMODE),
    ERR_INVITEONLYCHAN(responses::INVITEONLYCHAN),
    ERR_BANNEDFROMCHAN(responses::BannedFromChan),
    ERR_BADCHANNELKEY(responses::BadChannelKey),
    ERR_NOPRIVILEGES(responses::NOPRIVILEGES),
    ERR_CHANOPRIVSNEEDED(responses::CHANOPRIVSNEEDED),
    ERR_CANTKILLSERVER(responses::CANTKILLSERVER),
    ERR_NOOPERHOST(responses::NOOPERHOST),
    ERR_UMODEUNKNOWNFLAG(responses::UModeUnknownFlag),
    ERR_USERSDONTMATCH(responses::UsersDontMatch),
    // 6.2 Command responses.
    RPL_NONE(responses::NONE),
    RPL_USERHOST(responses::USERHOST),
    RPL_ISON(responses::ISON),
    RPL_AWAY(responses::AWAY),
    RPL_UNAWAY(responses::UNAWAY),
    RPL_NOWAWAY(responses::NOWAWAY),
    RPL_WHOISUSER(responses::WHOISUSER),
    RPL_WHOISSERVER(responses::WHOISSERVER),
    RPL_WHOISOPERATOR(responses::WHOISOPERATOR),
    RPL_WHOISIDLE(responses::WHOISIDLE),
    RPL_ENDOFWHOIS(responses::ENDOFWHOIS),
    RPL_WHOISCHANNELS(responses::WHOISCHANNELS),
    RPL_WHOWASUSER(responses::WHOWASUSER),
    RPL_ENDOFWHOWAS(responses::ENDOFWHOWAS),
    RPL_LISTSTART(responses::LISTSTART),
    RPL_LIST(responses::LIST),
    RPL_LISTEND(responses::LISTEND),
    RPL_CHANNELMODEIS(responses::CHANNELMODEIS),
    RPL_NOTOPIC(responses::NOTOPIC),
    RPL_TOPIC(responses::Topic),
    RPL_INVITING(responses::INVITING),
    RPL_SUMMONING(responses::SUMMONING),
    RPL_VERSION(responses::VERSION),
    RPL_WHOREPLY(responses::WHOREPLY),
    RPL_ENDOFWHO(responses::ENDOFWHO),
    RPL_NAMREPLY(responses::NamReply),
    RPL_ENDOFNAMES(responses::EndOfNames),
    RPL_LINKS(responses::LINKS),
    RPL_ENDOFLINKS(responses::ENDOFLINKS),
    RPL_BANLIST(responses::BANLIST),
    RPL_ENDOFBANLIST(responses::ENDOFBANLIST),
    RPL_INFO(responses::INFO),
    RPL_ENDOFINFO(responses::ENDOFINFO),
    RPL_MOTDSTART(responses::MOTDSTART),
    RPL_MOTD(responses::MOTD),
    RPL_ENDOFMOTD(responses::ENDOFMOTD),
    RPL_YOUREOPER(responses::YOUREOPER),
    RPL_REHASHING(responses::REHASHING),
    RPL_TIME(responses::TIME),
    RPL_USERSSTART(responses::USERSSTART),
    RPL_USERS(responses::USERS),
    RPL_ENDOFUSERS(responses::ENDOFUSERS),
    RPL_NOUSERS(responses::NOUSERS),
    RPL_TRACELINK(responses::TRACELINK),
    RPL_TRACECONNECTING(responses::TRACECONNECTING),
    RPL_TRACEHANDSHAKE(responses::TRACEHANDSHAKE),
    RPL_TRACEUNKNOWN(responses::TRACEUNKNOWN),
    RPL_TRACEOPERATOR(responses::TRACEOPERATOR),
    RPL_TRACEUSER(responses::TRACEUSER),
    RPL_TRACESERVER(responses::TRACESERVER),
    RPL_TRACENEWTYPE(responses::TRACENEWTYPE),
    RPL_TRACELOG(responses::TRACELOG),
    RPL_STATSLINKINFO(responses::STATSLINKINFO),
    RPL_STATSCOMMANDS(responses::STATSCOMMANDS),
    RPL_STATSCLINE(responses::STATSCLINE),
    RPL_STATSNLINE(responses::STATSNLINE),
    RPL_STATSILINE(responses::STATSILINE),
    RPL_STATSKLINE(responses::STATSKLINE),
    RPL_STATSYLINE(responses::STATSYLINE),
    RPL_ENDOFSTATS(responses::ENDOFSTATS),
    RPL_STATSLLINE(responses::STATSLLINE),
    RPL_STATSUPTIME(responses::STATSUPTIME),
    RPL_STATSOLINE(responses::STATSOLINE),
    RPL_STATSHLINE(responses::STATSHLINE),
    RPL_UMODEIS(responses::UMODEIS),
    RPL_LUSERCLIENT(responses::LUSERCLIENT),
    RPL_LUSEROP(responses::LUSEROP),
    RPL_LUSERUNKNOWN(responses::LUSERUNKNOWN),
    RPL_LUSERCHANNELS(responses::LUSERCHANNELS),
    RPL_LUSERME(responses::LUSERME),
    RPL_ADMINME(responses::ADMINME),
    RPL_ADMINLOC1(responses::ADMINLOC1),
    RPL_ADMINLOC2(responses::ADMINLOC2),
    RPL_ADMINEMAIL(responses::ADMINEMAIL),
    // 6.3 Reserved.
    RPL_TRACECLASS(responses::TRACECLASS),
    RPL_STATSQLINE(responses::STATSQLINE),
    RPL_SERVICEINFO(responses::SERVICEINFO),
    RPL_ENDOFSERVICES(responses::ENDOFSERVICES),
    RPL_SERVICE(responses::SERVICE),
    RPL_SERVLIST(responses::SERVLIST),
    RPL_SERVLISTEND(responses::SERVLISTEND),
    RPL_WHOISCHANOP(responses::WHOISCHANOP),
    RPL_KILLDONE(responses::KILLDONE),
    RPL_CLOSING(responses::CLOSING),
    RPL_CLOSEEND(responses::CLOSEEND),
    RPL_INFOSTART(responses::INFOSTART),
    RPL_MYPORTIS(responses::MYPORTIS),
    ERR_YOUWILLBEBANNED(responses::YOUWILLBEBANNED),
    ERR_BADCHANMASK(responses::BADCHANMASK),
    ERR_NOSERVICEHOST(responses::NOSERVICEHOST),
    // RFC 2812 5.1 Command responses.
    RPL_WELCOME(responses::Welcome),
    RPL_YOURHOST(responses::YourHost),
    RPL_CREATED(responses::Created),
    RPL_MYINFO(responses::MyInfo),
    RPL_ISUPPORT(responses::ISUPPORT),
    RPL_BOUNCE(responses::BOUNCE),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            &Command::NICK(ref c) => write!(f, "{}", c),
            &Command::PASS(ref c) => write!(f, "{}", c),
            &Command::USER(ref c) => write!(f, "{}", c),
            &Command::SERVER(ref c) => write!(f, "{}", c),
            &Command::OPER(ref c) => write!(f, "{}", c),
            &Command::SERVICE(ref c) => write!(f, "{}", c),
            &Command::QUIT(ref c) => write!(f, "{}", c),
            &Command::SQUIT(ref c) => write!(f, "{}", c),
            &Command::JOIN(ref c) => write!(f, "{}", c),
            &Command::PART(ref c) => write!(f, "{}", c),
            &Command::MODE(ref c) => write!(f, "{}", c),
            &Command::TOPIC(ref c) => write!(f, "{}", c),
            &Command::NAMES(ref c) => write!(f, "{}", c),
            &Command::LIST(ref c) => write!(f, "{}", c),
            &Command::INVITE(ref c) => write!(f, "{}", c),
            &Command::KICK(ref c) => write!(f, "{}", c),
            &Command::MOTD(ref c) => write!(f, "{}", c),
            &Command::LUSERS(ref c) => write!(f, "{}", c),
            &Command::VERSION(ref c) => write!(f, "{}", c),
            &Command::STATS(ref c) => write!(f, "{}", c),
            &Command::LINKS(ref c) => write!(f, "{}", c),
            &Command::TIME(ref c) => write!(f, "{}", c),
            &Command::CONNECT(ref c) => write!(f, "{}", c),
            &Command::TRACE(ref c) => write!(f, "{}", c),
            &Command::ADMIN(ref c) => write!(f, "{}", c),
            &Command::INFO(ref c) => write!(f, "{}", c),
            &Command::PRIVMSG(ref c) => write!(f, "{}", c),
            &Command::NOTICE(ref c) => write!(f, "{}", c),
            &Command::SERVLIST(ref c) => write!(f, "{}", c),
            &Command::SQUERY(ref c) => write!(f, "{}", c),
            &Command::WHO(ref c) => write!(f, "{}", c),
            &Command::WHOIS(ref c) => write!(f, "{}", c),
            &Command::WHOWAS(ref c) => write!(f, "{}", c),
            &Command::KILL(ref c) => write!(f, "{}", c),
            &Command::PING(ref c) => write!(f, "{}", c),
            &Command::PONG(ref c) => write!(f, "{}", c),
            &Command::ERROR(ref c) => write!(f, "{}", c),
            &Command::AWAY(ref c) => write!(f, "{}", c),
            &Command::REHASH(ref c) => write!(f, "{}", c),
            &Command::RESTART(ref c) => write!(f, "{}", c),
            &Command::SUMMON(ref c) => write!(f, "{}", c),
            &Command::USERS(ref c) => write!(f, "{}", c),
            &Command::WALLOPS(ref c) => write!(f, "{}", c),
            &Command::USERHOST(ref c) => write!(f, "{}", c),
            &Command::ISON(ref c) => write!(f, "{}", c),
            &Command::ERR_NOSUCHNICK(ref c) => write!(f, "{}", c),
            &Command::ERR_NOSUCHSERVER(ref c) => write!(f, "{}", c),
            &Command::ERR_NOSUCHCHANNEL(ref c) => write!(f, "{}", c),
            &Command::ERR_CANNOTSENDTOCHAN(ref c) => write!(f, "{}", c),
            &Command::ERR_TOOMANYCHANNELS(ref c) => write!(f, "{}", c),
            &Command::ERR_WASNOSUCHNICK(ref c) => write!(f, "{}", c),
            &Command::ERR_TOOMANYTARGETS(ref c) => write!(f, "{}", c),
            &Command::ERR_NOORIGIN(ref c) => write!(f, "{}", c),
            &Command::ERR_NORECIPIENT(ref c) => write!(f, "{}", c),
            &Command::ERR_NOTEXTTOSEND(ref c) => write!(f, "{}", c),
            &Command::ERR_NOTOPLEVEL(ref c) => write!(f, "{}", c),
            &Command::ERR_WILDTOPLEVEL(ref c) => write!(f, "{}", c),
            &Command::ERR_UNKNOWNCOMMAND(ref c) => write!(f, "{}", c),
            &Command::ERR_NOMOTD(ref c) => write!(f, "{}", c),
            &Command::ERR_NOADMININFO(ref c) => write!(f, "{}", c),
            &Command::ERR_FILEERROR(ref c) => write!(f, "{}", c),
            &Command::ERR_NONICKNAMEGIVEN(ref c) => write!(f, "{}", c),
            &Command::ERR_ERRONEUSNICKNAME(ref c) => write!(f, "{}", c),
            &Command::ERR_NICKNAMEINUSE(ref c) => write!(f, "{}", c),
            &Command::ERR_NICKCOLLISION(ref c) => write!(f, "{}", c),
            &Command::ERR_USERNOTINCHANNEL(ref c) => write!(f, "{}", c),
            &Command::ERR_NOTONCHANNEL(ref c) => write!(f, "{}", c),
            &Command::ERR_USERONCHANNEL(ref c) => write!(f, "{}", c),
            &Command::ERR_NOLOGIN(ref c) => write!(f, "{}", c),
            &Command::ERR_SUMMONDISABLED(ref c) => write!(f, "{}", c),
            &Command::ERR_USERSDISABLED(ref c) => write!(f, "{}", c),
            &Command::ERR_NOTREGISTERED(ref c) => write!(f, "{}", c),
            &Command::ERR_NEEDMOREPARAMS(ref c) => write!(f, "{}", c),
            &Command::ERR_ALREADYREGISTRED(ref c) => write!(f, "{}", c),
            &Command::ERR_NOPERMFORHOST(ref c) => write!(f, "{}", c),
            &Command::ERR_PASSWDMISMATCH(ref c) => write!(f, "{}", c),
            &Command::ERR_YOUREBANNEDCREEP(ref c) => write!(f, "{}", c),
            &Command::ERR_KEYSET(ref c) => write!(f, "{}", c),
            &Command::ERR_CHANNELISFULL(ref c) => write!(f, "{}", c),
            &Command::ERR_UNKNOWNMODE(ref c) => write!(f, "{}", c),
            &Command::ERR_INVITEONLYCHAN(ref c) => write!(f, "{}", c),
            &Command::ERR_BANNEDFROMCHAN(ref c) => write!(f, "{}", c),
            &Command::ERR_BADCHANNELKEY(ref c) => write!(f, "{}", c),
            &Command::ERR_NOPRIVILEGES(ref c) => write!(f, "{}", c),
            &Command::ERR_CHANOPRIVSNEEDED(ref c) => write!(f, "{}", c),
            &Command::ERR_CANTKILLSERVER(ref c) => write!(f, "{}", c),
            &Command::ERR_NOOPERHOST(ref c) => write!(f, "{}", c),
            &Command::ERR_UMODEUNKNOWNFLAG(ref c) => write!(f, "{}", c),
            &Command::ERR_USERSDONTMATCH(ref c) => write!(f, "{}", c),
            &Command::RPL_NONE(ref c) => write!(f, "{}", c),
            &Command::RPL_USERHOST(ref c) => write!(f, "{}", c),
            &Command::RPL_ISON(ref c) => write!(f, "{}", c),
            &Command::RPL_AWAY(ref c) => write!(f, "{}", c),
            &Command::RPL_UNAWAY(ref c) => write!(f, "{}", c),
            &Command::RPL_NOWAWAY(ref c) => write!(f, "{}", c),
            &Command::RPL_WHOISUSER(ref c) => write!(f, "{}", c),
            &Command::RPL_WHOISSERVER(ref c) => write!(f, "{}", c),
            &Command::RPL_WHOISOPERATOR(ref c) => write!(f, "{}", c),
            &Command::RPL_WHOISIDLE(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFWHOIS(ref c) => write!(f, "{}", c),
            &Command::RPL_WHOISCHANNELS(ref c) => write!(f, "{}", c),
            &Command::RPL_WHOWASUSER(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFWHOWAS(ref c) => write!(f, "{}", c),
            &Command::RPL_LISTSTART(ref c) => write!(f, "{}", c),
            &Command::RPL_LIST(ref c) => write!(f, "{}", c),
            &Command::RPL_LISTEND(ref c) => write!(f, "{}", c),
            &Command::RPL_CHANNELMODEIS(ref c) => write!(f, "{}", c),
            &Command::RPL_NOTOPIC(ref c) => write!(f, "{}", c),
            &Command::RPL_TOPIC(ref c) => write!(f, "{}", c),
            &Command::RPL_INVITING(ref c) => write!(f, "{}", c),
            &Command::RPL_SUMMONING(ref c) => write!(f, "{}", c),
            &Command::RPL_VERSION(ref c) => write!(f, "{}", c),
            &Command::RPL_WHOREPLY(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFWHO(ref c) => write!(f, "{}", c),
            &Command::RPL_NAMREPLY(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFNAMES(ref c) => write!(f, "{}", c),
            &Command::RPL_LINKS(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFLINKS(ref c) => write!(f, "{}", c),
            &Command::RPL_BANLIST(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFBANLIST(ref c) => write!(f, "{}", c),
            &Command::RPL_INFO(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFINFO(ref c) => write!(f, "{}", c),
            &Command::RPL_MOTDSTART(ref c) => write!(f, "{}", c),
            &Command::RPL_MOTD(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFMOTD(ref c) => write!(f, "{}", c),
            &Command::RPL_YOUREOPER(ref c) => write!(f, "{}", c),
            &Command::RPL_REHASHING(ref c) => write!(f, "{}", c),
            &Command::RPL_TIME(ref c) => write!(f, "{}", c),
            &Command::RPL_USERSSTART(ref c) => write!(f, "{}", c),
            &Command::RPL_USERS(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFUSERS(ref c) => write!(f, "{}", c),
            &Command::RPL_NOUSERS(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACELINK(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACECONNECTING(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACEHANDSHAKE(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACEUNKNOWN(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACEOPERATOR(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACEUSER(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACESERVER(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACENEWTYPE(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACELOG(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSLINKINFO(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSCOMMANDS(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSCLINE(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSNLINE(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSILINE(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSKLINE(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSYLINE(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFSTATS(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSLLINE(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSUPTIME(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSOLINE(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSHLINE(ref c) => write!(f, "{}", c),
            &Command::RPL_UMODEIS(ref c) => write!(f, "{}", c),
            &Command::RPL_LUSERCLIENT(ref c) => write!(f, "{}", c),
            &Command::RPL_LUSEROP(ref c) => write!(f, "{}", c),
            &Command::RPL_LUSERUNKNOWN(ref c) => write!(f, "{}", c),
            &Command::RPL_LUSERCHANNELS(ref c) => write!(f, "{}", c),
            &Command::RPL_LUSERME(ref c) => write!(f, "{}", c),
            &Command::RPL_ADMINME(ref c) => write!(f, "{}", c),
            &Command::RPL_ADMINLOC1(ref c) => write!(f, "{}", c),
            &Command::RPL_ADMINLOC2(ref c) => write!(f, "{}", c),
            &Command::RPL_ADMINEMAIL(ref c) => write!(f, "{}", c),
            &Command::RPL_TRACECLASS(ref c) => write!(f, "{}", c),
            &Command::RPL_STATSQLINE(ref c) => write!(f, "{}", c),
            &Command::RPL_SERVICEINFO(ref c) => write!(f, "{}", c),
            &Command::RPL_ENDOFSERVICES(ref c) => write!(f, "{}", c),
            &Command::RPL_SERVICE(ref c) => write!(f, "{}", c),
            &Command::RPL_SERVLIST(ref c) => write!(f, "{}", c),
            &Command::RPL_SERVLISTEND(ref c) => write!(f, "{}", c),
            &Command::RPL_WHOISCHANOP(ref c) => write!(f, "{}", c),
            &Command::RPL_KILLDONE(ref c) => write!(f, "{}", c),
            &Command::RPL_CLOSING(ref c) => write!(f, "{}", c),
            &Command::RPL_CLOSEEND(ref c) => write!(f, "{}", c),
            &Command::RPL_INFOSTART(ref c) => write!(f, "{}", c),
            &Command::RPL_MYPORTIS(ref c) => write!(f, "{}", c),
            &Command::ERR_YOUWILLBEBANNED(ref c) => write!(f, "{}", c),
            &Command::ERR_BADCHANMASK(ref c) => write!(f, "{}", c),
            &Command::ERR_NOSERVICEHOST(ref c) => write!(f, "{}", c),
            &Command::RPL_WELCOME(ref c) => write!(f, "{}", c),
            &Command::RPL_YOURHOST(ref c) => write!(f, "{}", c),
            &Command::RPL_CREATED(ref c) => write!(f, "{}", c),
            &Command::RPL_MYINFO(ref c) => write!(f, "{}", c),
            &Command::RPL_ISUPPORT(ref c) => write!(f, "{}", c),
            &Command::RPL_BOUNCE(ref c) => write!(f, "{}", c),
        }
    }
}

fn extract_params<'a>(
    rem: &'a str,
    required: usize,
    err: &'static str,
) -> Result<Vec<&'a str>, ParseError> {
    let mut rem = rem;
    let mut params: Vec<&str> = Vec::new();
    while rem.len() > 0 {
        if rem.starts_with(':') {
            if rem.len() == 1 {
                warn!("Empty trailing command parameter. Ignoring.")
            } else {
                params.push(&rem[1..]);
            }
            break;
        }

        let (next_param, r) = next_token(rem);
        rem = r;

        if next_param.len() == 0 {
            warn!("Empty whitespace in parameters list: ignoring.");
        } else {
            params.push(next_param);
        }
    }

    if params.len() < required {
        return Err(ParseError::NeedMoreParams { command: err.to_string() });
    }

    Ok(params)
}

// Macro to generate a value for a required field.
macro_rules! rf {
    ($p:expr, $idx:expr, $type:ty) => { $p[$idx].parse::<$type>()? };
}

// Macro to generate a value for as optional field.
// Should only be used if the the optional field is at the end of the param list.
macro_rules! of {
    ($p:expr, $idx:expr, $type:ty) => {
        if $p.len() > $idx {
            Some($p[$idx].parse::<$type>()?)
        } else {
            None
        }
    };
}

impl str::FromStr for Command {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (command, r) = next_token(s);

        match command.to_uppercase().as_ref() {
            // Unfortunately, Rust's macros aren't powerful enough to capture variants:
            // https://stackoverflow.com/questions/37006835/building-an-enum-inside-a-macro.
            "NICK" => {
                let p = try!(extract_params(r, 1, "NICK"));
                Ok(Command::NICK(
                    requests::Nick { nickname: rf!(p, 0, String) },
                ))
            }

            "PASS" => {
                let p = try!(extract_params(r, 1, "PASS"));
                Ok(Command::PASS(
                    requests::Pass { password: rf!(p, 0, String) },
                ))
            }

            "USER" => {
                let p = try!(extract_params(r, 4, "USER"));
                Ok(Command::USER(requests::User {
                    username: rf!(p, 0, String),
                    mode: rf!(p, 1, String),
                    unused: rf!(p, 2, String),
                    realname: rf!(p, 3, String),
                }))
            }

            "SERVER" => {
                let p = try!(extract_params(r, 4, "SERVER"));
                Ok(Command::SERVER(requests::Server {
                    servername: rf!(p, 0, String),
                    hopcount: rf!(p, 1, u64),
                    token: rf!(p, 2, u64),
                    info: rf!(p, 3, String),
                }))
            }

            "SERVICE" => {
                let p = try!(extract_params(r, 6, "SERVICE"));
                Ok(Command::SERVICE(requests::Service {
                    nickname: rf!(p, 0, String),
                    reserved1: rf!(p, 1, String),
                    distribution: rf!(p, 2, String),
                    ty: rf!(p, 3, String),
                    reserved2: rf!(p, 4, String),
                    info: rf!(p, 5, String),
                }))
            }

            "OPER" => {
                let p = try!(extract_params(r, 2, "OPER"));
                Ok(Command::OPER(requests::Oper {
                    name: rf!(p, 0, String),
                    password: rf!(p, 1, String),
                }))
            }

            "QUIT" => {
                let p = try!(extract_params(r, 0, "QUIT"));
                Ok(Command::QUIT(requests::Quit { message: of!(p, 0, String) }))
            }

            "SQUIT" => {
                let p = try!(extract_params(r, 2, "SQUIT"));
                Ok(Command::SQUIT(requests::Squit {
                    server: rf!(p, 0, String),
                    comment: rf!(p, 1, String),
                }))
            }

            "JOIN" => {
                let p = try!(extract_params(r, 1, "JOIN"));
                let chan = rf!(p, 0, String);
                if p.len() == 1 && chan == "0" {
                    Ok(Command::JOIN(
                        requests::Join { join: requests::JoinChannels::PartAll },
                    ))
                } else if p.len() > 1 {
                    let chan: Vec<String> = chan.split(",").map(|s| s.to_string()).collect();
                    let keys: Vec<String> = rf!(p, 1, String)
                        .split(",")
                        .map(|s| s.to_string())
                        .collect();
                    if keys.len() != chan.len() {
                        return Err(ParseError::NeedMoreParams { command: "JOIN".to_string() });
                    }
                    Ok(Command::JOIN(requests::Join {
                        join: requests::JoinChannels::KeyedChannels(
                            chan.into_iter().zip(keys).collect(),
                        ),
                    }))
                } else {
                    Ok(Command::JOIN(requests::Join {
                        join: requests::JoinChannels::Channels(
                            chan.split(",").map(|s| s.to_string()).collect(),
                        ),
                    }))
                }
            }

            "PART" => {
                let p = try!(extract_params(r, 1, "PART"));
                Ok(Command::PART(requests::Part {
                    channels: rf!(p, 0, String)
                        .split(",")
                        .map(|s| s.to_string())
                        .collect(),
                    message: of!(p, 1, String),
                }))
            }

            "MODE" => {
                let p = try!(extract_params(r, 1, "MODE"));
                Ok(Command::MODE(requests::Mode {
                    target: rf!(p, 0, String),
                    mode_string: of!(p, 1, String),
                    mode_args: if p.len() > 2 {
                        Some(p[2..].join(" "))
                    } else {
                        None
                    },
                }))
            }

            "TOPIC" => {
                let p = try!(extract_params(r, 1, "TOPIC"));
                Ok(Command::TOPIC(requests::Topic {
                    channel: rf!(p, 0, String),
                    topic: of!(p, 1, String),
                }))
            }

            "NAMES" => {
                let p = try!(extract_params(r, 0, "NAMES"));
                Ok(Command::NAMES(requests::Names {
                    channels: rf!(p, 0, String)
                        .split(",")
                        .map(|s| s.to_string())
                        .collect(),
                }))
            }

            "LIST" => {
                let p = try!(extract_params(r, 0, "LIST"));
                Ok(Command::LIST(requests::List {
                    channels: of!(p, 0, String).map_or(Vec::new(), |s| {
                        s.split(",").map(|s| s.to_string()).collect()
                    }),
                    elist: of!(p, 1, String).map_or(Vec::new(), |s| {
                        s.split(",").map(|s| s.to_string()).collect()
                    }),
                }))
            }

            "INVITE" => {
                let p = try!(extract_params(r, 2, "INVITE"));
                Ok(Command::INVITE(requests::Invite {
                    nickname: rf!(p, 0, String),
                    channel: rf!(p, 1, String),
                }))
            }

            "KICK" => {
                let p = try!(extract_params(r, 2, "KICK"));
                Ok(Command::KICK(requests::Kick {
                    channels: rf!(p, 0, String)
                        .split(",")
                        .map(|s| s.to_string())
                        .collect(),
                    users: rf!(p, 1, String)
                        .split(",")
                        .map(|s| s.to_string())
                        .collect(),
                    comment: of!(p, 2, String),
                }))
            }

            "MOTD" => {
                let p = try!(extract_params(r, 0, "MOTD"));
                Ok(Command::MOTD(requests::Motd { target: of!(p, 0, String) }))
            }

            "LUSERS" => {
                let p = try!(extract_params(r, 0, "MOTD"));
                Ok(Command::LUSERS(requests::Lusers {
                    mask: of!(p, 0, String),
                    target: of!(p, 1, String),
                }))
            }

            "VERSION" => {
                let p = try!(extract_params(r, 0, "VERSION"));
                Ok(Command::VERSION(
                    requests::Version { target: of!(p, 0, String) },
                ))
            }

            "STATS" => {
                let p = try!(extract_params(r, 0, "STATS"));
                Ok(Command::STATS(requests::Stats {
                    query: of!(p, 0, requests::StatsQuery),
                    target: of!(p, 1, String),
                }))
            }

            "LINKS" => {
                let p = try!(extract_params(r, 0, "LINKS"));

                let (mut remote, mut mask) = (None, None);
                if p.len() > 1 {
                    remote = of!(p, 0, String);
                    mask = of!(p, 1, String);
                } else if p.len() == 1 {
                    mask = of!(p, 0, String);
                }

                Ok(Command::LINKS(requests::Links {
                    remote_server: remote,
                    server_mask: mask,
                }))
            }

            "TIME" => {
                let p = try!(extract_params(r, 0, "TIME"));
                Ok(Command::TIME(requests::Time { target: of!(p, 0, String) }))
            }

            "CONNECT" => {
                let p = try!(extract_params(r, 1, "CONNECT"));
                Ok(Command::CONNECT(requests::Connect {
                    target: rf!(p, 0, String),
                    port: of!(p, 1, u32),
                    remote: of!(p, 2, String),
                }))
            }

            "TRACE" => {
                let p = try!(extract_params(r, 0, "TRACE"));
                Ok(Command::TRACE(
                    requests::Trace { target: of!(p, 0, String) },
                ))
            }

            "ADMIN" => {
                let p = try!(extract_params(r, 0, "ADMIN"));
                Ok(Command::ADMIN(
                    requests::Admin { target: of!(p, 0, String) },
                ))
            }

            "INFO" => {
                let p = try!(extract_params(r, 0, "INFO"));
                Ok(Command::INFO(requests::Info { target: of!(p, 0, String) }))
            }

            "PRIVMSG" => {
                let p = try!(extract_params(r, 2, "PRIVMSG"));
                Ok(Command::PRIVMSG(requests::Privmsg {
                    targets: rf!(p, 0, String)
                        .split(",")
                        .map(|s| s.to_string())
                        .collect(),
                    message: rf!(p, 1, String),
                }))
            }

            "NOTICE" => {
                let p = try!(extract_params(r, 2, "NOTICE"));
                Ok(Command::NOTICE(requests::Notice {
                    targets: rf!(p, 0, String)
                        .split(",")
                        .map(|s| s.to_string())
                        .collect(),
                    message: rf!(p, 1, String),
                }))
            }

            "SERVLIST" => {
                let p = try!(extract_params(r, 0, "SERVLIST"));
                Ok(Command::SERVLIST(requests::Servlist {
                    mask: of!(p, 0, String),
                    server_type: of!(p, 1, String),
                }))
            }

            "SQUERY" => {
                let p = try!(extract_params(r, 2, "SQUERY"));
                Ok(Command::SQUERY(requests::Squery {
                    servicename: rf!(p, 0, String),
                    text: rf!(p, 1, String),
                }))
            }

            "WHO" => {
                let p = try!(extract_params(r, 0, "WHO"));
                let mut mask = None;
                let mut oper = false;
                if p.len() > 0 {
                    mask = Some(p[0].to_string());
                    if p.len() > 1 {
                        oper = p[1] == "o";
                    }
                }
                Ok(Command::WHO(requests::Who {
                    mask: mask,
                    operators: oper,
                }))
            }

            "WHOIS" => {
                let p = try!(extract_params(r, 1, "WHOIS"));

                let parsed;
                if p.len() == 1 {
                    parsed = Command::WHOIS(requests::Whois {
                        target: None,
                        masks: rf!(p, 0, String)
                            .split(",")
                            .map(|s| s.to_string())
                            .collect(),
                    });
                } else {
                    // p.len() > 1
                    parsed = Command::WHOIS(requests::Whois {
                        target: of!(p, 0, String),
                        masks: rf!(p, 1, String)
                            .split(",")
                            .map(|s| s.to_string())
                            .collect(),
                    });
                }
                Ok(parsed)
            }

            "WHOWAS" => {
                let p = try!(extract_params(r, 1, "WHOWAS"));
                Ok(Command::WHOWAS(requests::Whowas {
                    nicknames: rf!(p, 0, String)
                        .split(",")
                        .map(|s| s.to_string())
                        .collect(),
                    max: of!(p, 1, i64),
                    target: of!(p, 2, String),
                }))
            }

            "KILL" => {
                let p = try!(extract_params(r, 2, "KILL"));
                Ok(Command::KILL(requests::Kill {
                    nickname: rf!(p, 0, String),
                    comment: rf!(p, 1, String),
                }))
            }

            "PING" => {
                let p = try!(extract_params(r, 1, "PING"));
                Ok(Command::PING(requests::Ping {
                    originator: rf!(p, 0, String),
                    target: of!(p, 1, String),
                }))
            }

            "PONG" => {
                let p = try!(extract_params(r, 1, "PONG"));
                Ok(Command::PONG(requests::Pong {
                    originator: rf!(p, 0, String),
                    target: of!(p, 1, String),
                }))
            }

            "ERROR" => {
                let p = try!(extract_params(r, 1, "ERROR"));
                Ok(Command::ERROR(
                    requests::Error { message: rf!(p, 0, String) },
                ))
            }

            "AWAY" => {
                let p = try!(extract_params(r, 0, "AWAY"));
                Ok(Command::AWAY(requests::Away { message: of!(p, 1, String) }))
            }

            "REHASH" => Ok(Command::REHASH(requests::Rehash {})),

            "RESTART" => Ok(Command::RESTART(requests::Restart {})),

            "SUMMON" => {
                let p = try!(extract_params(r, 1, "SUMMON"));
                Ok(Command::SUMMON(requests::Summon {
                    user: rf!(p, 0, String),
                    target: of!(p, 1, String),
                    channel: of!(p, 2, String),
                }))
            }

            "USERS" => {
                let p = try!(extract_params(r, 0, "USERS"));
                Ok(Command::USERS(
                    requests::Users { target: of!(p, 0, String) },
                ))
            }

            "WALLOPS" => {
                let p = try!(extract_params(r, 1, "WALLOPS"));
                Ok(Command::WALLOPS(
                    requests::Wallops { message: rf!(p, 0, String) },
                ))
            }

            "USERHOST" => {
                let p = try!(extract_params(r, 1, "USERHOST"));
                Ok(Command::USERHOST(requests::Userhost {
                    nicknames: p.into_iter().map(|s| s.to_string()).collect(),
                }))
            }

            "ISON" => {
                let p = try!(extract_params(r, 1, "ISON"));
                Ok(Command::ISON(requests::Ison {
                    nicknames: p.into_iter().map(|s| s.to_string()).collect(),
                }))
            }

            // TODO(lazau): Parse parameters.
            "401" => Ok(Command::ERR_NOSUCHNICK(responses::NOSUCHNICK::default())),
            "402" => Ok(Command::ERR_NOSUCHSERVER(
                responses::NOSUCHSERVER::default(),
            )),
            "403" => Ok(Command::ERR_NOSUCHCHANNEL(
                responses::NoSuchChannel::default(),
            )),
            "404" => Ok(Command::ERR_CANNOTSENDTOCHAN(
                responses::CANNOTSENDTOCHAN::default(),
            )),
            "405" => Ok(Command::ERR_TOOMANYCHANNELS(
                responses::TOOMANYCHANNELS::default(),
            )),
            "406" => Ok(Command::ERR_WASNOSUCHNICK(
                responses::WASNOSUCHNICK::default(),
            )),
            "407" => Ok(Command::ERR_TOOMANYTARGETS(
                responses::TOOMANYTARGETS::default(),
            )),
            "409" => Ok(Command::ERR_NOORIGIN(responses::NOORIGIN::default())),
            "411" => Ok(Command::ERR_NORECIPIENT(responses::NORECIPIENT::default())),
            "412" => Ok(Command::ERR_NOTEXTTOSEND(
                responses::NOTEXTTOSEND::default(),
            )),
            "413" => Ok(Command::ERR_NOTOPLEVEL(responses::NOTOPLEVEL::default())),
            "414" => Ok(Command::ERR_WILDTOPLEVEL(
                responses::WILDTOPLEVEL::default(),
            )),
            "421" => Ok(Command::ERR_UNKNOWNCOMMAND(
                responses::UNKNOWNCOMMAND::default(),
            )),
            "422" => Ok(Command::ERR_NOMOTD(responses::NOMOTD::default())),
            "423" => Ok(Command::ERR_NOADMININFO(responses::NOADMININFO::default())),
            "424" => Ok(Command::ERR_FILEERROR(responses::FILEERROR::default())),
            "431" => Ok(Command::ERR_NONICKNAMEGIVEN(
                responses::NONICKNAMEGIVEN::default(),
            )),
            "432" => Ok(Command::ERR_ERRONEUSNICKNAME(
                responses::ERRONEUSNICKNAME::default(),
            )),
            "433" => Ok(Command::ERR_NICKNAMEINUSE(
                responses::NICKNAMEINUSE::default(),
            )),
            "436" => Ok(Command::ERR_NICKCOLLISION(
                responses::NICKCOLLISION::default(),
            )),
            "441" => Ok(Command::ERR_USERNOTINCHANNEL(
                responses::USERNOTINCHANNEL::default(),
            )),
            "442" => Ok(Command::ERR_NOTONCHANNEL(
                responses::NotOnChannel::default(),
            )),
            "443" => Ok(Command::ERR_USERONCHANNEL(
                responses::USERONCHANNEL::default(),
            )),
            "444" => Ok(Command::ERR_NOLOGIN(responses::NOLOGIN::default())),
            "445" => Ok(Command::ERR_SUMMONDISABLED(
                responses::SUMMONDISABLED::default(),
            )),
            "446" => Ok(Command::ERR_USERSDISABLED(
                responses::USERSDISABLED::default(),
            )),
            "451" => Ok(Command::ERR_NOTREGISTERED(
                responses::NOTREGISTERED::default(),
            )),
            "461" => Ok(Command::ERR_NEEDMOREPARAMS(
                responses::NeedMoreParams::default(),
            )),
            "462" => Ok(Command::ERR_ALREADYREGISTRED(
                responses::AlreadyRegistered::default(),
            )),
            "463" => Ok(Command::ERR_NOPERMFORHOST(
                responses::NOPERMFORHOST::default(),
            )),
            "464" => Ok(Command::ERR_PASSWDMISMATCH(
                responses::PASSWDMISMATCH::default(),
            )),
            "465" => Ok(Command::ERR_YOUREBANNEDCREEP(
                responses::YOUREBANNEDCREEP::default(),
            )),
            "467" => Ok(Command::ERR_KEYSET(responses::KEYSET::default())),
            "471" => Ok(Command::ERR_CHANNELISFULL(
                responses::CHANNELISFULL::default(),
            )),
            "472" => Ok(Command::ERR_UNKNOWNMODE(responses::UNKNOWNMODE::default())),
            "473" => Ok(Command::ERR_INVITEONLYCHAN(
                responses::INVITEONLYCHAN::default(),
            )),
            "474" => Ok(Command::ERR_BANNEDFROMCHAN(
                responses::BannedFromChan::default(),
            )),
            "475" => Ok(Command::ERR_BADCHANNELKEY(
                responses::BadChannelKey::default(),
            )),
            "481" => Ok(Command::ERR_NOPRIVILEGES(
                responses::NOPRIVILEGES::default(),
            )),
            "482" => Ok(Command::ERR_CHANOPRIVSNEEDED(
                responses::CHANOPRIVSNEEDED::default(),
            )),
            "483" => Ok(Command::ERR_CANTKILLSERVER(
                responses::CANTKILLSERVER::default(),
            )),
            "491" => Ok(Command::ERR_NOOPERHOST(responses::NOOPERHOST::default())),
            "501" => Ok(Command::ERR_UMODEUNKNOWNFLAG(
                responses::UModeUnknownFlag::default(),
            )),
            "502" => Ok(Command::ERR_USERSDONTMATCH(
                responses::UsersDontMatch::default(),
            )),
            "300" => Ok(Command::RPL_NONE(responses::NONE::default())),
            "302" => Ok(Command::RPL_USERHOST(responses::USERHOST::default())),
            "303" => Ok(Command::RPL_ISON(responses::ISON::default())),
            "301" => Ok(Command::RPL_AWAY(responses::AWAY::default())),
            "305" => Ok(Command::RPL_UNAWAY(responses::UNAWAY::default())),
            "306" => Ok(Command::RPL_NOWAWAY(responses::NOWAWAY::default())),
            "311" => Ok(Command::RPL_WHOISUSER(responses::WHOISUSER::default())),
            "312" => Ok(Command::RPL_WHOISSERVER(responses::WHOISSERVER::default())),
            "313" => Ok(Command::RPL_WHOISOPERATOR(
                responses::WHOISOPERATOR::default(),
            )),
            "317" => Ok(Command::RPL_WHOISIDLE(responses::WHOISIDLE::default())),
            "318" => Ok(Command::RPL_ENDOFWHOIS(responses::ENDOFWHOIS::default())),
            "319" => Ok(Command::RPL_WHOISCHANNELS(
                responses::WHOISCHANNELS::default(),
            )),
            "314" => Ok(Command::RPL_WHOWASUSER(responses::WHOWASUSER::default())),
            "369" => Ok(Command::RPL_ENDOFWHOWAS(responses::ENDOFWHOWAS::default())),
            "321" => Ok(Command::RPL_LISTSTART(responses::LISTSTART::default())),
            "322" => Ok(Command::RPL_LIST(responses::LIST::default())),
            "323" => Ok(Command::RPL_LISTEND(responses::LISTEND::default())),
            "324" => Ok(Command::RPL_CHANNELMODEIS(
                responses::CHANNELMODEIS::default(),
            )),
            "331" => Ok(Command::RPL_NOTOPIC(responses::NOTOPIC::default())),
            "332" => Ok(Command::RPL_TOPIC(responses::Topic::default())),
            "341" => Ok(Command::RPL_INVITING(responses::INVITING::default())),
            "342" => Ok(Command::RPL_SUMMONING(responses::SUMMONING::default())),
            "351" => Ok(Command::RPL_VERSION(responses::VERSION::default())),
            "352" => Ok(Command::RPL_WHOREPLY(responses::WHOREPLY::default())),
            "315" => Ok(Command::RPL_ENDOFWHO(responses::ENDOFWHO::default())),
            "353" => Ok(Command::RPL_NAMREPLY(responses::NamReply::default())),
            "366" => Ok(Command::RPL_ENDOFNAMES(responses::EndOfNames::default())),
            "364" => Ok(Command::RPL_LINKS(responses::LINKS::default())),
            "365" => Ok(Command::RPL_ENDOFLINKS(responses::ENDOFLINKS::default())),
            "367" => Ok(Command::RPL_BANLIST(responses::BANLIST::default())),
            "368" => Ok(Command::RPL_ENDOFBANLIST(
                responses::ENDOFBANLIST::default(),
            )),
            "371" => Ok(Command::RPL_INFO(responses::INFO::default())),
            "374" => Ok(Command::RPL_ENDOFINFO(responses::ENDOFINFO::default())),
            "375" => Ok(Command::RPL_MOTDSTART(responses::MOTDSTART::default())),
            "372" => Ok(Command::RPL_MOTD(responses::MOTD::default())),
            "376" => Ok(Command::RPL_ENDOFMOTD(responses::ENDOFMOTD::default())),
            "381" => Ok(Command::RPL_YOUREOPER(responses::YOUREOPER::default())),
            "382" => Ok(Command::RPL_REHASHING(responses::REHASHING::default())),
            "391" => Ok(Command::RPL_TIME(responses::TIME::default())),
            "392" => Ok(Command::RPL_USERSSTART(responses::USERSSTART::default())),
            "393" => Ok(Command::RPL_USERS(responses::USERS::default())),
            "394" => Ok(Command::RPL_ENDOFUSERS(responses::ENDOFUSERS::default())),
            "395" => Ok(Command::RPL_NOUSERS(responses::NOUSERS::default())),
            "200" => Ok(Command::RPL_TRACELINK(responses::TRACELINK::default())),
            "201" => Ok(Command::RPL_TRACECONNECTING(
                responses::TRACECONNECTING::default(),
            )),
            "202" => Ok(Command::RPL_TRACEHANDSHAKE(
                responses::TRACEHANDSHAKE::default(),
            )),
            "203" => Ok(Command::RPL_TRACEUNKNOWN(
                responses::TRACEUNKNOWN::default(),
            )),
            "204" => Ok(Command::RPL_TRACEOPERATOR(
                responses::TRACEOPERATOR::default(),
            )),
            "205" => Ok(Command::RPL_TRACEUSER(responses::TRACEUSER::default())),
            "206" => Ok(Command::RPL_TRACESERVER(responses::TRACESERVER::default())),
            "208" => Ok(Command::RPL_TRACENEWTYPE(
                responses::TRACENEWTYPE::default(),
            )),
            "261" => Ok(Command::RPL_TRACELOG(responses::TRACELOG::default())),
            "211" => Ok(Command::RPL_STATSLINKINFO(
                responses::STATSLINKINFO::default(),
            )),
            "212" => Ok(Command::RPL_STATSCOMMANDS(
                responses::STATSCOMMANDS::default(),
            )),
            "213" => Ok(Command::RPL_STATSCLINE(responses::STATSCLINE::default())),
            "214" => Ok(Command::RPL_STATSNLINE(responses::STATSNLINE::default())),
            "215" => Ok(Command::RPL_STATSILINE(responses::STATSILINE::default())),
            "216" => Ok(Command::RPL_STATSKLINE(responses::STATSKLINE::default())),
            "218" => Ok(Command::RPL_STATSYLINE(responses::STATSYLINE::default())),
            "219" => Ok(Command::RPL_ENDOFSTATS(responses::ENDOFSTATS::default())),
            "241" => Ok(Command::RPL_STATSLLINE(responses::STATSLLINE::default())),
            "242" => Ok(Command::RPL_STATSUPTIME(responses::STATSUPTIME::default())),
            "243" => Ok(Command::RPL_STATSOLINE(responses::STATSOLINE::default())),
            "244" => Ok(Command::RPL_STATSHLINE(responses::STATSHLINE::default())),
            "221" => Ok(Command::RPL_UMODEIS(responses::UMODEIS::default())),
            "251" => Ok(Command::RPL_LUSERCLIENT(responses::LUSERCLIENT::default())),
            "252" => Ok(Command::RPL_LUSEROP(responses::LUSEROP::default())),
            "253" => Ok(Command::RPL_LUSERUNKNOWN(
                responses::LUSERUNKNOWN::default(),
            )),
            "254" => Ok(Command::RPL_LUSERCHANNELS(
                responses::LUSERCHANNELS::default(),
            )),
            "255" => Ok(Command::RPL_LUSERME(responses::LUSERME::default())),
            "256" => Ok(Command::RPL_ADMINME(responses::ADMINME::default())),
            "257" => Ok(Command::RPL_ADMINLOC1(responses::ADMINLOC1::default())),
            "258" => Ok(Command::RPL_ADMINLOC2(responses::ADMINLOC2::default())),
            "259" => Ok(Command::RPL_ADMINEMAIL(responses::ADMINEMAIL::default())),
            "209" => Ok(Command::RPL_TRACECLASS(responses::TRACECLASS::default())),
            "217" => Ok(Command::RPL_STATSQLINE(responses::STATSQLINE::default())),
            "231" => Ok(Command::RPL_SERVICEINFO(responses::SERVICEINFO::default())),
            "232" => Ok(Command::RPL_ENDOFSERVICES(
                responses::ENDOFSERVICES::default(),
            )),
            "233" => Ok(Command::RPL_SERVICE(responses::SERVICE::default())),
            "234" => Ok(Command::RPL_SERVLIST(responses::SERVLIST::default())),
            "235" => Ok(Command::RPL_SERVLISTEND(responses::SERVLISTEND::default())),
            "316" => Ok(Command::RPL_WHOISCHANOP(responses::WHOISCHANOP::default())),
            "361" => Ok(Command::RPL_KILLDONE(responses::KILLDONE::default())),
            "362" => Ok(Command::RPL_CLOSING(responses::CLOSING::default())),
            "363" => Ok(Command::RPL_CLOSEEND(responses::CLOSEEND::default())),
            "373" => Ok(Command::RPL_INFOSTART(responses::INFOSTART::default())),
            "384" => Ok(Command::RPL_MYPORTIS(responses::MYPORTIS::default())),
            "466" => Ok(Command::ERR_YOUWILLBEBANNED(
                responses::YOUWILLBEBANNED::default(),
            )),
            "476" => Ok(Command::ERR_BADCHANMASK(responses::BADCHANMASK::default())),
            "492" => Ok(Command::ERR_NOSERVICEHOST(
                responses::NOSERVICEHOST::default(),
            )),
            "001" => Ok(Command::RPL_WELCOME(responses::Welcome::default())),
            "002" => Ok(Command::RPL_YOURHOST(responses::YourHost::default())),
            "003" => Ok(Command::RPL_CREATED(responses::Created::default())),
            "004" => Ok(Command::RPL_MYINFO(responses::MyInfo::default())),
            "005" => Ok(Command::RPL_ISUPPORT(responses::ISUPPORT::default())),
            "010" => Ok(Command::RPL_BOUNCE(responses::BOUNCE::default())),
            _ => Err(ParseError::UnrecognizedCommand),
        }
    }
}
