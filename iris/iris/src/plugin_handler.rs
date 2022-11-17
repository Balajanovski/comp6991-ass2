use std::collections::{BTreeMap};

use common::plugin::{PluginMod_Ref, RPluginReply};
use common::types::{PluginName, PluginMsg, PluginReply, ErrorType, Nick};
use std::path::Path;
use log::{error, warn};
use anyhow::anyhow;


pub struct PluginHandler {
    plugins: BTreeMap<PluginName, PluginMod_Ref>,
}

impl PluginHandler {
    pub fn new(plugin_paths: &Vec<String>) -> PluginHandler {
        let plugins = plugin_paths.iter().map(|path| {
            let plugin = abi_stable::library::lib_header_from_path(
                Path::new(path)
            ).and_then(|header| header.init_root_module::<PluginMod_Ref>());

            plugin
        }).filter_map(|p_res| {
            match p_res {
                Ok(p) => Some(p),
                Err(err) => {
                    error!("{err}");
                    None
                },
            }
        }).collect::<Vec<_>>();

        info!(
            "Session Loaded Plugins: {:?}", 
            plugins.iter().map(|pl| { pl.pl_name()().into() }).collect::<Vec<PluginName>>()
        );

        let mut plugin_map = BTreeMap::new();
        for pl in plugins {
            let pl_name = PluginName::from(pl.pl_name()());
            pl.init()();

            if plugin_map.contains_key(&pl_name) {
                warn!("Plugin with name {} already loaded. Overwriting...", &pl_name)
            }

            plugin_map.insert(pl_name, pl);
        }

        PluginHandler { plugins: plugin_map }
    }

    pub fn handle(&self, nick: &Nick, real_name: &String, plugin_msg: PluginMsg) -> anyhow::Result<Option<PluginReply>> {
        let pl_name = plugin_msg.plugin_name.clone();
        let plugin = self.plugins.get(&pl_name)
            .ok_or(ErrorType::NoSuchPlugin)
            .map_err(|e| anyhow!(e))?;
        
        let pl_repl = Result::from(plugin.handler()(nick.clone().into(), real_name.clone().into(), plugin_msg.into()))
            .map_err(|e| {
                error!("Plugin (Name: {}) Exception: {}", &pl_name, String::from(e));
                anyhow!(ErrorType::PluginException)
            });

        match pl_repl {
            Ok(pl_repl) => Ok(Option::from(pl_repl).map(|repl: RPluginReply| repl.into())),
            Err(_) => Ok(None),
        }
    }
}
