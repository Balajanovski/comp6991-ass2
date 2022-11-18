//! # An example plugin
//!
//! ## Introduction
//! Plugins are loaded into IRIS utilising dynamic loading.
//! Creating the plugin interface is done using the stable_abi crate.
//! This blog series goes into the approach well: https://nullderef.com/blog/plugin-dynload/
//!
//! ## Loading
//! When the plugin is built, it will output a dynamic library file (.so extension).
//! To run IRIS with the plugin loaded, simply run:
//! > cargo +nightly run -- --plugins '/path/to/the/plugin.so'

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{ROption, RResult, RString},
};

use common::plugin::{
    PluginMod, PluginMod_Ref, RNick, RPluginMsg, RPluginName, RPluginReply, RTarget,
};

/// # Plugin Initialisation
/// This function is run on plugin startup.
/// It can be used to start up any initial required state.
#[sabi_extern_fn]
pub fn init() {
    // We require no initialisation
    // So we leave it empty
}

/// # Plugin Name
/// This function returns a string which IRIS will listen for to
/// know to run this plugin.
/// It MUST start with a '/'.
#[sabi_extern_fn]
pub fn pl_name() -> RPluginName {
    RPluginName(RString::from("/example"))
}

/// # Plugin Handler
/// This function will be run whenever the plugin command is typed.
/// For example: `PLUGIN /example :hi`.
/// It will be passed in the nickname of the user who ran the command, as well as the arguments they passed.
/// You return a Result (errors as string can be raised in case input is invalid).
/// The Result contains an Optional reply. This will be sent to the appropriate target.
/// If you do not want your plugin to output anything, simply have this optional be None.
#[sabi_extern_fn]
pub fn handler(
    sender: RNick,
    real_name: RString,
    msg: RPluginMsg,
) -> RResult<ROption<RPluginReply>, RString> {
    if msg.args.len() != 1 {
        RResult::RErr(RString::from("Expected 1 argument"))
    } else {
        RResult::ROk(ROption::RSome(RPluginReply {
            target: RTarget::RUser(sender),
            message: format!("Echo \"{}\" to \"{}\"", msg.args[0].clone(), real_name).into(),
        }))
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
