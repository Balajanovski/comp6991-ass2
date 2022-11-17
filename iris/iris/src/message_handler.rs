//! # Message handler
//! Implements state design pattern to handle transitions between handler states
//! Very loosely based off of: https://hoverbear.org/blog/rust-state-machine-pattern/

use common::connect::{ConnectionError, ConnectionWrite};
use common::types::*;
use crate::plugin_handler::PluginHandler;
use crate::user_connections::UserConnections;
use log::{error, info};
use std::sync::{Arc, Mutex};

pub enum ClientState {
    Fresh(Fresh),
    Nicked(Nicked),
    Initialised(Initialised),
    Quit,
}

pub struct Fresh {
    curr_writer: Arc<Mutex<ConnectionWrite>>,
}

pub struct Nicked {
    nick: Nick,
}

pub struct Initialised {
    real_name: String,
    nick: Nick,
}

pub struct MessageHandler {
    state: ClientState,
    user_connections: Arc<Mutex<UserConnections>>,
    plugin_handler: PluginHandler,
}

impl MessageHandler {
    pub fn new(
        user_connections: Arc<Mutex<UserConnections>>,
        curr_writer: ConnectionWrite,
        plugin_paths: Vec<String>,
    ) -> MessageHandler {
        MessageHandler {
            state: ClientState::Fresh(Fresh {
                curr_writer: Arc::new(Mutex::new(curr_writer)),
            }),
            user_connections,
            plugin_handler: PluginHandler::new(&plugin_paths),
        }
    }

    pub fn handle(&mut self, message: anyhow::Result<String>) {
        match self.transition(message) {
            Ok(_) => {}
            Err(err) => {
                // Handle any uncaught errors by quitting the session

                error!("{err}");

                if let Some(nick) = self.get_nick() {
                    let mut user_conn_guard = self.user_connections.lock().unwrap();
                    user_conn_guard.remove_user(&nick);
                }

                self.state = ClientState::Quit;
            }
        }
    }

    pub fn has_quit(&self) -> bool {
        let has_quit = match self.state {
            ClientState::Quit => true,
            _ => false,
        };

        has_quit
    }

    fn transition(&mut self, message: anyhow::Result<String>) -> anyhow::Result<()> {
        let message = message.as_deref().map(ParsedMessage::try_from);
        let message = match message {
            Ok(Ok(message)) => message,
            Err(err) => match err.downcast_ref::<ConnectionError>() {
                Some(ConnectionError::ConnectionLost | ConnectionError::ConnectionClosed) => {
                    info!("Lost connection.");

                    if let Some(nick) = self.get_nick() {
                        let mut user_conn_guard = self.user_connections.lock().unwrap();
                        user_conn_guard.remove_user(&nick);
                    }

                    self.state = ClientState::Quit;
                    return Ok(());
                }
                Some(_) | None => {
                    error!("Invalid message received... ignoring message.");

                    return Ok(());
                }
            },
            Ok(Err(err)) => {
                error!("{err}");

                return Ok(());
            }
        };

        self.transition_parsed(message.message)
    }

    fn transition_parsed(&mut self, message: Message) -> anyhow::Result<()> {
        match (&self.state, message) {
            (ClientState::Fresh(state), Message::Nick(nick_msg)) => {
                let nick = nick_msg.nick.clone();
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                user_conn_guard.add_user(&nick, state.curr_writer.clone())?;

                self.state = ClientState::Nicked(Nicked { nick });
            }
            (ClientState::Nicked(state), Message::User(user_msg)) => {
                let real_name = user_msg.real_name.clone();
                let mut user_conn_guard = self.user_connections.lock().unwrap();

                let nick = state.nick.clone();
                user_conn_guard.write_to_user(
                    &nick,
                    &Reply::Welcome(WelcomeReply {
                        target_nick: nick.clone(),
                        message: format!("Hi {real_name}, welcome to IRC"),
                    }),
                )?;

                self.state = ClientState::Initialised(Initialised { nick, real_name });
            }
            (ClientState::Initialised(state), Message::Ping(ping_msg)) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                let nick = state.nick.clone();
                user_conn_guard.write_to_user(&nick, &Reply::Pong(ping_msg.clone()))?;
            }
            (ClientState::Initialised(state), Message::Quit(quit_msg)) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                let nick = state.nick.clone();
                user_conn_guard.write_to_users_channel(
                    &nick,
                    &Reply::Quit(QuitReply {
                        message: quit_msg.clone(),
                        sender_nick: nick.clone(),
                    }),
                )?;

                info!("{nick} has quit...");
                user_conn_guard.remove_user(&nick);

                self.state = ClientState::Quit;
            }
            (ClientState::Initialised(state), Message::PrivMsg(priv_msg)) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                let nick = state.nick.clone();
                user_conn_guard.write(
                    &priv_msg.target,
                    &Reply::PrivMsg(PrivReply {
                        message: priv_msg.clone(),
                        sender_nick: nick.clone(),
                    }),
                )?;
            }
            (ClientState::Initialised(state), Message::Join(join_msg)) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                let nick = state.nick.clone();
                user_conn_guard.add_user_to_channel(&nick, &join_msg.channel)?;
                user_conn_guard.write_to_channel(
                    &join_msg.channel,
                    &Reply::Join(JoinReply {
                        message: join_msg.clone(),
                        sender_nick: nick.clone(),
                    }),
                )?;
            }
            (ClientState::Initialised(state), Message::Part(part_msg)) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                let nick = state.nick.clone();
                user_conn_guard.remove_user_from_channel(&nick, &part_msg.channel)?;
                user_conn_guard.write_to_channel(
                    &part_msg.channel,
                    &Reply::Part(PartReply {
                        message: part_msg.clone(),
                        sender_nick: nick.clone(),
                    }),
                )?;
            }
            (ClientState::Initialised(state), Message::Plugin(plugin_msg)) => {
                let nick = state.nick.clone();
                let plugin_reply = self.plugin_handler.handle(&nick, &state.real_name, plugin_msg)?;

                if let Some(plugin_reply) = plugin_reply {
                    let mut user_conn_guard = self.user_connections.lock().unwrap();
                    user_conn_guard.write(&plugin_reply.target.clone(), &Reply::Plugin(plugin_reply))?;
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn get_nick(&self) -> Option<Nick> {
        let nick = match &self.state {
            ClientState::Nicked(state) => Some(state.nick.clone()),
            ClientState::Initialised(state) => Some(state.nick.clone()),
            _ => None,
        };

        nick
    }
}
