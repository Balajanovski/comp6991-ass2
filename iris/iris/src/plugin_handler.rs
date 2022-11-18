use std::collections::BTreeMap;

use crate::user_connections::UserConnections;
use closure::closure;
use common::plugin::PluginMod_Ref;
use common::types::{ErrorType, Nick, PluginMsg, PluginName, PluginReply, Reply};
use log::{error, warn};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct PluginHandler {
    plugins: Arc<Mutex<BTreeMap<PluginName, PluginMod_Ref>>>,
    user_connections: Arc<Mutex<UserConnections>>,
}

impl PluginHandler {
    pub fn new(
        plugin_paths: &Vec<String>,
        user_connections: Arc<Mutex<UserConnections>>,
    ) -> PluginHandler {
        let plugins = plugin_paths
            .iter()
            .map(|path| {
                let plugin = abi_stable::library::lib_header_from_path(Path::new(path))
                    .and_then(|header| header.init_root_module::<PluginMod_Ref>());

                plugin
            })
            .filter_map(|p_res| match p_res {
                Ok(p) => Some(p),
                Err(err) => {
                    error!("{err}");
                    None
                }
            })
            .collect::<Vec<_>>();

        info!(
            "Session Loaded Plugins: {:?}",
            plugins
                .iter()
                .map(|pl| { pl.pl_name()().into() })
                .collect::<Vec<PluginName>>()
        );

        let mut plugin_map = BTreeMap::new();
        for pl in plugins {
            let pl_name = PluginName::from(pl.pl_name()());
            pl.init()();

            if plugin_map.contains_key(&pl_name) {
                warn!(
                    "Plugin with name {} already loaded. Overwriting...",
                    &pl_name
                )
            }

            plugin_map.insert(pl_name, pl);
        }

        PluginHandler {
            plugins: Arc::new(Mutex::new(plugin_map)),
            user_connections,
        }
    }

    pub fn handle(&self, nick: &Nick, real_name: &String, plugin_msg: PluginMsg) {
        let pl_name = plugin_msg.plugin_name.clone();
        let plugins = self.plugins.clone();
        let nick = nick.clone();
        let real_name = real_name.clone();
        let user_connections = self.user_connections.clone();

        // Run a detached thread with the plugin
        // This is to allow the plugin to implement delays
        // without slowing the server down
        thread::spawn(
            closure!(move pl_name, move plugins, move nick, move real_name, move user_connections, || {
                let plugins_guard = plugins.lock().unwrap();
                let plugin = plugins_guard.get(&pl_name)
                    .ok_or(ErrorType::NoSuchPlugin)
                    .map_err(|_| {
                        let error_str = format!("Plugin {} not found\r\n", &pl_name);
                        error!("{error_str}");

                        let mut user_conn_guard = user_connections.lock().unwrap();
                        let _ = user_conn_guard.write_to_user(&nick, &error_str);

                        return;
                    });

                if let Ok(plugin) = plugin {
                    let plugin_reply = Result::from(plugin.handler()(nick.clone().into(), real_name.clone().into(), plugin_msg.into()))
                        .map_err(|e| {
                            let error_str = format!("Plugin (Name: {}) Exception: {}\r\n", &pl_name, e);
                            error!("{error_str}");

                            let mut user_conn_guard = user_connections.lock().unwrap();
                            let _ = user_conn_guard.write_to_user(&nick, &error_str);

                            return;
                        });

                    if let Ok(plugin_reply) = plugin_reply {
                        let plugin_reply = Option::<PluginReply>::from(plugin_reply.map(|repl| repl.into()));

                        if let Some(plugin_reply) = plugin_reply {
                            let mut user_conn_guard = user_connections.lock().unwrap();

                            // We ignore any errors when writing, as if a plugin's output gets lost, it is not mission critical
                            let _ = user_conn_guard.write(&plugin_reply.target.clone(), &Reply::Plugin(plugin_reply).to_string());
                        }
                    }
                }
            }),
        );
    }
}
