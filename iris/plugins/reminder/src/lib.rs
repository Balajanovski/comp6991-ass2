use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RString, ROption, RResult},
};

use std::{thread, time};

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
pub fn init() { }

#[sabi_extern_fn]
pub fn pl_name() -> RPluginName {
    RPluginName(RString::from("/remind"))
}

#[sabi_extern_fn]
pub fn handler(sender: RNick, _: RString, msg: RPluginMsg) -> RResult<ROption<RPluginReply>, RString> {
    if msg.args.len() != 2 {
        RResult::RErr(RString::from("Expected 2 arguments. Ex: PLUGIN /remind {interval in seconds} :message"))
    } else {
        let interval = match msg.args[0].to_string().parse::<u64>() {
            Ok(interval) => interval,
            Err(_) => { return RResult::RErr(RString::from("Please provide a valid integer interval")); },
        };

        let message = msg.args[1].to_string();
        thread::sleep(time::Duration::from_secs(interval));

        RResult::ROk(
            ROption::RSome(
                RPluginReply {
                    target: RTarget::RUser(sender),
                    message: format!("Reminder: {}", &message).into(),
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