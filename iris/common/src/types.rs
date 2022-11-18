use crate::plugin::{RChannel, RNick, RPluginMsg, RPluginName, RPluginReply, RTarget};

/// All relevant IRC errors are listed here.
/// See the assignment documentation for more information.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ErrorType {
    NoNickNameGiven = 431,
    ErroneousNickname = 432,
    NickCollision = 436,
    NoRecipient = 411,
    NoTextToSend = 412,
    NoOrigin = 409,
    UnknownCommand = 421,
    NeedMoreParams = 461,
    NoSuchNick = 401,
    NoSuchChannel = 403,
    PluginException = 998,
    NoSuchPlugin = 999,
}

/// This is the name of your server, all messages originating from
/// the server should be listed as from this name.
pub const SERVER_NAME: &str = "iris-server";

impl std::fmt::Display for ErrorType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            ErrorType::NoNickNameGiven => {
                write!(fmt, ":{SERVER_NAME} 431 :No nickname given.")
            }
            ErrorType::ErroneousNickname => {
                // Typo is same as in RFC1459
                write!(fmt, ":{SERVER_NAME} 432 :Erroneus nickname")
            }
            ErrorType::NoRecipient => {
                write!(fmt, ":{SERVER_NAME} 411 :No recipient given")
            }
            ErrorType::NoTextToSend => {
                write!(fmt, ":{SERVER_NAME} 412 :No text to send")
            }
            ErrorType::NoOrigin => {
                write!(fmt, ":{SERVER_NAME} 409 :No origin specified")
            }
            ErrorType::UnknownCommand => {
                write!(fmt, ":{SERVER_NAME} 421 :Unknown command")
            }
            ErrorType::NeedMoreParams => {
                write!(fmt, ":{SERVER_NAME} 461 :Not enough parameters")
            }
            ErrorType::NoSuchNick => {
                write!(fmt, ":{SERVER_NAME} 401 :No such nick/channel")
            }
            ErrorType::NoSuchChannel => {
                write!(fmt, ":{SERVER_NAME} 403 :No such channel")
            }
            ErrorType::NickCollision => {
                write!(fmt, ":{SERVER_NAME} 436 :Nickname collision")
            }
            ErrorType::PluginException => {
                write!(fmt, ":{SERVER_NAME} 998 :Plugin exception")
            }
            ErrorType::NoSuchPlugin => {
                write!(fmt, ":{SERVER_NAME} 999 :No such plugin")
            }
        }
    }
}

/// Given an IRC command, this will split it up into component parts.
/// Particularly, the prefix (optionally), then all space-separated args,
/// then (optionally) the final argument.
fn split_command(cmd: &str) -> Vec<&str> {
    let stripped = cmd.strip_suffix("\r\n").unwrap_or(cmd);

    match stripped.split_once(':') {
        Some((before, after)) => {
            let mut cmd_vec = before.split(' ').collect::<Vec<_>>();

            if cmd_vec.last() == Some(&"") {
                let _ = cmd_vec.pop();
            }

            cmd_vec.push(after.trim_start());
            cmd_vec
        }
        None => stripped.split(' ').collect::<Vec<_>>(),
    }
}

/// A person or channel to whom a command is addressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Channel(Channel),
    User(Nick),
}

impl From<String> for Target {
    fn from(value: String) -> Self {
        if value.starts_with('#') {
            Target::Channel(Channel(value))
        } else {
            Target::User(Nick(value))
        }
    }
}

impl From<RTarget> for Target {
    fn from(target: RTarget) -> Self {
        match target {
            RTarget::RChannel(channel) => Target::Channel(channel.into()),
            RTarget::RUser(user) => Target::User(user.into()),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Target::Channel(s) => write!(fmt, "{s}"),
            Target::User(s) => write!(fmt, "{s}"),
        }
    }
}

/// A nickname.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Nick(pub String);

impl TryFrom<String> for Nick {
    type Error = ErrorType;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value;
        if (1..10).contains(&value.len())
            && value.is_ascii()
            && value.chars().next().unwrap_or('!').is_alphabetic()
            && value.chars().all(char::is_alphanumeric)
        {
            Ok(Nick(value))
        } else {
            Err(ErrorType::ErroneousNickname)
        }
    }
}

impl From<RNick> for Nick {
    fn from(nick: RNick) -> Self {
        Nick(nick.0.into())
    }
}

impl std::fmt::Display for Nick {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(fmt)
    }
}

/// An IRC channel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Channel(pub String);

impl TryFrom<String> for Channel {
    type Error = ErrorType;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if (1..200).contains(&value.len())
            && value.chars().next().unwrap_or('!') == '#'
            && value.is_ascii()
            && value[1..].chars().all(char::is_alphanumeric)
        {
            Ok(Channel(value))
        } else {
            Err(ErrorType::NoSuchChannel)
        }
    }
}

impl From<RChannel> for Channel {
    fn from(channel: RChannel) -> Self {
        Channel(channel.0.into())
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(fmt)
    }
}

/// An IRC plugin name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PluginName(pub String);

impl TryFrom<String> for PluginName {
    type Error = ErrorType;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if (1..20).contains(&value.len())
            && value.chars().next().unwrap_or('!') == '/'
            && value.is_ascii()
            && value[1..].chars().all(char::is_alphanumeric)
        {
            Ok(PluginName(value))
        } else {
            Err(ErrorType::NoSuchPlugin)
        }
    }
}

impl From<RPluginName> for PluginName {
    fn from(name: RPluginName) -> Self {
        PluginName(name.0.into())
    }
}

impl std::fmt::Display for PluginName {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

/// A message to set the nickname.
/// For example: `NICK tfpk\r\n`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NickMsg {
    pub nick: Nick,
}

impl TryFrom<Vec<String>> for NickMsg {
    type Error = ErrorType;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .nth(1)
            .ok_or(ErrorType::NoNickNameGiven)
            .and_then(|value| Nick::try_from(value))
            .map(|nick| NickMsg { nick })
    }
}

/// A message to join a channel.
/// For example: `JOIN #channel\r\n`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinMsg {
    pub channel: Channel,
}

impl TryFrom<Vec<String>> for JoinMsg {
    type Error = ErrorType;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .nth(1)
            .ok_or(ErrorType::NeedMoreParams)
            .and_then(|value| Channel::try_from(value))
            .map(|channel| JoinMsg { channel })
    }
}

/// A message to leave a channel.
/// For example: `PART #channel\r\n`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartMsg {
    pub channel: Channel,
}

impl TryFrom<Vec<String>> for PartMsg {
    type Error = ErrorType;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .nth(1)
            .ok_or(ErrorType::NeedMoreParams)
            .and_then(|value| Channel::try_from(value))
            .map(|channel| PartMsg { channel })
    }
}

/// A message to register a new user.
// For example: `USER ignored ignored ignored :Thomas Kunc\r\n`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserMsg {
    pub real_name: String,
}

impl TryFrom<Vec<String>> for UserMsg {
    type Error = ErrorType;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .nth(4)
            .ok_or(ErrorType::NeedMoreParams)
            .map(|real_name| UserMsg { real_name })
    }
}

/// A plugin message
/// For example: `PLUGIN /remind 10 :message`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMsg {
    pub plugin_name: PluginName,
    pub args: Vec<String>,
}

impl TryFrom<Vec<String>> for PluginMsg {
    type Error = ErrorType;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        Ok(PluginMsg {
            plugin_name: PluginName::try_from(
                value.get(1).ok_or(ErrorType::NoSuchPlugin)?.to_string(),
            )?,
            args: (&value[2..value.len()]).to_vec(),
        })
    }
}

impl From<RPluginMsg> for PluginMsg {
    fn from(msg: RPluginMsg) -> Self {
        PluginMsg {
            plugin_name: msg.plugin_name.into(),
            args: msg.args.into_iter().map(|arg| arg.into()).collect(),
        }
    }
}

/// A private message.
/// For example: `PRIVMSG tom :Hi Tom, how are you?\r\n`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivMsg {
    pub target: Target,
    pub message: String,
}

impl TryFrom<Vec<String>> for PrivMsg {
    type Error = ErrorType;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        Ok(PrivMsg {
            target: Target::from(value.get(1).ok_or(ErrorType::NoRecipient)?.to_string()),
            // skip(2) here skips the PRIVMSG instruction and target.
            message: value
                .into_iter()
                .skip(2)
                .last()
                .ok_or(ErrorType::NoTextToSend)?,
        })
    }
}

/// The last message a user will send before leaving.
/// For example: `QUIT :Leaving now!`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuitMsg {
    pub message: Option<String>,
}

impl TryFrom<Vec<String>> for QuitMsg {
    type Error = ErrorType;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        Ok(QuitMsg {
            // skip(1) here skips the QUIT instruction.
            message: value.into_iter().skip(1).last(),
        })
    }
}

/// A list of every possible message that can be sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Nick(NickMsg),
    User(UserMsg),
    PrivMsg(PrivMsg),
    Ping(String),
    Join(JoinMsg),
    Part(PartMsg),
    Quit(QuitMsg),
    Plugin(PluginMsg),
}

/// To parse a message, construct this struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnparsedMessage<'a> {
    pub message: &'a str,
}

impl<'a> From<&'a str> for UnparsedMessage<'a> {
    fn from(value: &'a str) -> Self {
        UnparsedMessage { message: value }
    }
}

/// After parsing an `UnparsedMessage`, this struct will be created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMessage {
    pub message: Message,
}

impl<'a> TryFrom<UnparsedMessage<'a>> for ParsedMessage {
    type Error = ErrorType;
    fn try_from(value: UnparsedMessage<'a>) -> Result<Self, Self::Error> {
        let command = split_command(value.message)
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        let message = match command[0].as_str() {
            "PING" => Ok(Message::Ping(
                // Skip here ignores the "PING".
                command
                    .into_iter()
                    .skip(1)
                    .last()
                    .ok_or(ErrorType::NoOrigin)?
                    .to_string(),
            )),
            "PRIVMSG" => Ok(Message::PrivMsg(PrivMsg::try_from(command)?)),
            "USER" => Ok(Message::User(UserMsg::try_from(command)?)),
            "NICK" => Ok(Message::Nick(NickMsg::try_from(command)?)),
            "JOIN" => Ok(Message::Join(JoinMsg::try_from(command)?)),
            "PART" => Ok(Message::Part(PartMsg::try_from(command)?)),
            "QUIT" => Ok(Message::Quit(QuitMsg::try_from(command)?)),
            "PLUGIN" => Ok(Message::Plugin(PluginMsg::try_from(command)?)),
            _ => Err(ErrorType::UnknownCommand),
        }?;

        Ok(ParsedMessage { message })
    }
}

impl<'a> TryFrom<&'a str> for ParsedMessage {
    type Error = ErrorType;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        ParsedMessage::try_from(UnparsedMessage::from(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivReply {
    pub message: PrivMsg,
    pub sender_nick: Nick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinReply {
    pub message: JoinMsg,
    pub sender_nick: Nick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartReply {
    pub message: PartMsg,
    pub sender_nick: Nick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuitReply {
    pub message: QuitMsg,
    pub sender_nick: Nick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WelcomeReply {
    pub target_nick: Nick,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginReply {
    pub target: Target,
    pub message: String,
}

impl From<RPluginReply> for PluginReply {
    fn from(reply: RPluginReply) -> Self {
        PluginReply {
            target: reply.target.into(),
            message: reply.message.into(),
        }
    }
}

/// Every possible reply to a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    Pong(String),
    Welcome(WelcomeReply),
    PrivMsg(PrivReply),
    Join(JoinReply),
    Part(PartReply),
    Error(ErrorType),
    Quit(QuitReply),
    Plugin(PluginReply),
}

impl std::fmt::Display for Reply {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Reply::Pong(p) => write!(fmt, "PONG :{p}\r\n"),
            Reply::Plugin(p) => {
                let target = &p.target;
                let message = &p.message;
                write!(fmt, "PLUGIN {target} : {message}\r\n")
            }
            Reply::Welcome(r) => {
                let nick = &r.target_nick;
                let message = &r.message;
                write!(fmt, ":{SERVER_NAME} 001 {nick} :{message}\r\n")
            }
            Reply::PrivMsg(r) => {
                let nick = &r.message.target;
                let message = &r.message.message;
                let from = &r.sender_nick;
                write!(fmt, ":{from} PRIVMSG {nick} :{message}\r\n")
            }
            Reply::Error(e) => {
                write!(fmt, ":{SERVER_NAME} {e}\r\n")
            }
            Reply::Join(r) => {
                let sender = &r.sender_nick;
                let channel = &r.message.channel;
                write!(fmt, ":{sender} JOIN {channel}\r\n")
            }
            Reply::Part(r) => {
                let sender = &r.sender_nick;
                let channel = &r.message.channel;
                write!(fmt, ":{sender} PART {channel}\r\n")
            }
            Reply::Quit(r) => {
                let sender = &r.sender_nick.to_string();
                let message = &r.message.message.as_ref().unwrap_or(sender);
                write!(fmt, ":{sender} QUIT :{message}\r\n")
            }
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_ping() {
        assert_eq!(
            ParsedMessage::try_from(UnparsedMessage {
                message: "PING :host-name with space\r\n",
            })
            .unwrap()
            .message,
            Message::Ping("host-name with space".to_string())
        )
    }

    #[test]
    fn test_privmsg() {
        assert_eq!(
            ParsedMessage::try_from(UnparsedMessage {
                message: "PRIVMSG tom :Hi Tom, how are you?\r\n",
            })
            .unwrap()
            .message,
            Message::PrivMsg(PrivMsg {
                target: Target::User(Nick("tom".to_string())),
                message: "Hi Tom, how are you?".to_string()
            })
        )
    }

    #[test]
    fn test_nick() {
        assert_eq!(
            ParsedMessage::try_from(UnparsedMessage {
                message: "NICK tfpk\r\n",
            })
            .unwrap()
            .message,
            Message::Nick(NickMsg {
                nick: Nick("tfpk".to_string())
            })
        );
        assert_eq!(
            ParsedMessage::try_from(UnparsedMessage {
                message: "NICK tfpkasdfasdfasdf\r\n",
            }),
            Err(ErrorType::ErroneousNickname)
        );
    }

    #[test]
    fn test_plugin() {
        assert_eq!(
            ParsedMessage::try_from(UnparsedMessage {
                message: "PLUGIN /example hi :hello world\r\n",
            })
            .unwrap()
            .message,
            Message::Plugin(PluginMsg {
                plugin_name: PluginName::try_from("/example".to_string()).unwrap(),
                args: vec!["hi".to_string(), "hello world".to_string()],
            })
        );
        assert_eq!(
            ParsedMessage::try_from(UnparsedMessage {
                message: "PLUGIN djwaodkwaldj\r\n",
            }),
            Err(ErrorType::NoSuchPlugin)
        );
    }
}
