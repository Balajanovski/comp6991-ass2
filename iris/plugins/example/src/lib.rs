use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RString, ROption, RResult},
};

use common::plugin::{
    PluginMod, 
    RPluginName, 
    RPluginReply, 
    RPluginMsg, 
    RTarget,
    RNick,
    PluginMod_Ref,
};

#[sabi_extern_fn]
pub fn init() {

}

#[sabi_extern_fn]
pub fn pl_name() -> RPluginName {
    RPluginName(RString::from("/example"))
}

#[sabi_extern_fn]
pub fn handler(sender: RNick, msg: RPluginMsg) -> RResult<ROption<RPluginReply>, RString> {
    if msg.short_args.len() > 0 {
        RResult::RErr(RString::from("Too many arguments"))
    } else {
        RResult::ROk(
            ROption::RSome(
                RPluginReply {
                    target: RTarget::RUser(sender),
                    message: msg.long_arg,
                }
            )
        )
    }
}

#[export_root_module]
fn instantiate_root_module() -> PluginMod_Ref {
    PluginMod {
        init,
        pl_name,
        handler,
    }
    .leak_into_prefix()
}