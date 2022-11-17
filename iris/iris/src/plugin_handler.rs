use std::collections::{BTreeMap};

use common::plugin::{PluginMod_Ref, RPluginReply};
use common::types::{PluginName, PluginMsg, PluginReply, ErrorType, Nick};
use std::path::Path;
use log::error;
use anyhow::anyhow;


pub struct PluginHandler {
    plugins: BTreeMap<PluginName, PluginMod_Ref>,
}

impl PluginHandler {
    pub fn new(plugin_paths: &Vec<String>) -> anyhow::Result<PluginHandler> {
        let plugins = plugin_paths.iter().map(|path| {
            let plugin = abi_stable::library::lib_header_from_path(
                Path::new(path)
            ).and_then(|header| header.init_root_module::<PluginMod_Ref>());

            plugin
        }).collect::<Result<Vec<_>, _>>()?;

        let mut plugin_map = BTreeMap::new();
        for pl in plugins {
            plugin_map.insert(pl.pl_name()().into(), pl);
        }

        Ok(PluginHandler { plugins: plugin_map })
    }

    pub fn handle(&self, nick: &Nick, plugin_msg: PluginMsg) -> anyhow::Result<Option<PluginReply>> {
        let plugin = self.plugins.get(&plugin_msg.plugin_name)
            .ok_or(ErrorType::NoSuchPlugin)
            .map_err(|e| anyhow!(e))?;
        
        let pl_repl = Result::from(plugin.handler()(nick.clone().into(), plugin_msg.into()))
            .map_err(|e| {
                error!("Plugin Exception: {}", String::from(e));
                anyhow!(ErrorType::PluginException)
            })?;

        Ok(Option::from(pl_repl).map(|repl: RPluginReply| repl.into()))
    }
}
