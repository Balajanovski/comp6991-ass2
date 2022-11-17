use abi_stable::{
    declare_root_module_statics,
    library::{LibraryError, RootModule},
    package_version_strings,
    sabi_types::VersionStrings,
    std_types::{RString, ROption, RVec, RResult},
    StableAbi,
};

use crate::types::{PluginReply, PluginMsg, PluginName, Nick, Channel, Target};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "PluginMod_Ref")))]
#[sabi(missing_field(panic))]
pub struct PluginMod {
    pub init: extern "C" fn(),
    pub pl_name: extern "C" fn() -> RPluginName,
    pub handler: extern "C" fn(sender: RNick, message: RPluginMsg) -> RResult<ROption<RPluginReply>, RString>,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct RNick(pub RString);

impl From<Nick> for RNick {
    fn from(nick: Nick) -> Self {
        RNick(nick.0.into())
    }
}

#[repr(C)]
#[derive(StableAbi)]
pub struct RChannel(pub RString);

impl From<Channel> for RChannel {
    fn from(channel: Channel) -> Self {
        RChannel(channel.0.into())
    }
}

#[repr(C)]
#[derive(StableAbi)]
pub enum RTarget {
    RChannel(RChannel),
    RUser(RNick),
}

#[repr(C)]
#[derive(StableAbi)]
pub struct RPluginName(pub RString);

impl From<PluginName> for RPluginName {
    fn from(name: PluginName) -> Self {
        RPluginName(name.0.into())
    }
}

#[repr(C)]
#[derive(StableAbi)]
pub struct RPluginReply {
    pub target: RTarget,
    pub message: RString,
}

impl From<PluginReply> for RPluginReply {
    fn from(repl: PluginReply) -> Self {
        let target = match repl.target {
            Target::Channel(channel) => RTarget::RChannel(channel.into()),
            Target::User(user) => RTarget::RUser(user.into()),
        };

        RPluginReply { target, message: repl.message.into() }
    }
}

#[repr(C)]
#[derive(StableAbi)]
pub struct RPluginMsg {
    pub plugin_name: RPluginName,
    pub short_args: RVec<RString>,
    pub long_arg: RString,
}

impl From<PluginMsg> for RPluginMsg {
    fn from(repl: PluginMsg) -> Self {
        RPluginMsg { 
            plugin_name: repl.plugin_name.into(), 
            short_args: repl.short_args.into_iter().map(|v| v.into()).collect(), 
            long_arg: repl.long_arg.into(),
        }
    }
}

impl RootModule for PluginMod_Ref {
    declare_root_module_statics! {PluginMod_Ref}
    const BASE_NAME: &'static str = "iris";
    const NAME: &'static str = "iris";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();

    fn initialization(self) -> Result<Self, LibraryError> {
        Ok(self)
    }
}