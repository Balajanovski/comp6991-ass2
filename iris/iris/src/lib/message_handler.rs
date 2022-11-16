//! # Connection State Storage
//! Stores various information about the connection
//! Implements state design pattern to handle transitions
//! See: https://refactoring.guru/design-patterns/state/rust/example

use crate::types::{Nick, Message, Reply, WelcomeReply, PrivMsg, PrivReply, ErrorType, ParsedMessage, QuitMsg, QuitReply};
use crate::user_connections::UserConnections;
use crate::connect::ConnectionWrite;
use std::sync::{Arc, Mutex, MutexGuard};
use log::{error};

/// The base connection state interface
pub trait MessageHandler {
    /// Handle an incoming message
    fn handle(self: Box<Self>, message: &Message) -> Box<dyn MessageHandler>;

    fn has_quit(&self) -> bool {
        false
    }
}

/// A freshly started connection state
pub struct FreshHandler {
    user_connections: Arc<Mutex<UserConnections>>,
    curr_writer: ConnectionWrite,
}

impl FreshHandler {
    pub fn new(user_connections: Arc<Mutex<UserConnections>>, curr_writer: ConnectionWrite) -> Box<FreshHandler> {
        Box::new(FreshHandler { user_connections, curr_writer })
    }
}

impl MessageHandler for FreshHandler {
    fn handle(mut self: Box<Self>, message: &Message) -> Box<dyn MessageHandler> {
        match message {
            Message::Nick(nick_msg) => {
                let nick = nick_msg.nick.clone();
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                if let Err(err) = user_conn_guard.add_user(&nick, self.curr_writer) {
                    handle_fatal_error(&nick, &mut user_conn_guard, err)
                } else {
                    Box::new(NickedHandler { user_connections: self.user_connections.clone(), nick })
                }
            },
            _ => self,
        }
    }
}

/// A connection state with a nick
pub struct NickedHandler {
    user_connections: Arc<Mutex<UserConnections>>,
    nick: Nick,
}

impl MessageHandler for NickedHandler {
    fn handle(self: Box<Self>, message: &Message) -> Box<dyn MessageHandler> {
        match message {
            Message::User(user_msg) => {
                let real_name = user_msg.real_name.clone();

                let mut user_conn_guard = self.user_connections.lock().unwrap();
                if let Err(err) = user_conn_guard.write_to_user(
                    &self.nick, 
                    &Reply::Welcome(
                        WelcomeReply {
                            target_nick: self.nick.clone(),
                            message: format!("Hi {real_name}, welcome to IRC"),
                        }
                    )
                ) {
                    handle_fatal_error(&self.nick, &mut user_conn_guard, err)
                } else {
                    Box::new(InitialisedHandler { 
                        user_connections: self.user_connections.clone(), 
                        nick: self.nick, 
                        real_name,
                    })
                }
            },
            _ => self,
        }
    }
}

/// Fully initialised connection
pub struct InitialisedHandler {
    user_connections: Arc<Mutex<UserConnections>>,
    nick: Nick,
    real_name: String,
}

impl MessageHandler for InitialisedHandler {
    fn handle(self: Box<Self>, message: &Message) -> Box<dyn MessageHandler> {
        match message {
            Message::Ping(ping_msg) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                if let Err(err) = user_conn_guard.write_to_user(
                    &self.nick, 
                    &Reply::Pong(ping_msg.clone()),
                ) {
                    handle_fatal_error(&self.nick, &mut user_conn_guard, err)
                } else {
                    drop(user_conn_guard);
                    self
                }
            },
            Message::Quit(quit_msg) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                if let Err(err) = user_conn_guard.write_to_users_channel(
                    &self.nick, 
                    &Reply::Quit(QuitReply { message: quit_msg.clone(), sender_nick: self.nick.clone() }),
                ) {
                    handle_fatal_error(&self.nick, &mut user_conn_guard, err)
                } else {
                    user_conn_guard.remove_user(&self.nick);
                    Box::new(QuitHandler)
                }
            },
            _ => self,
        }
    }
}

/// The session has quit
pub struct QuitHandler;

impl MessageHandler for QuitHandler {
    fn handle(self: Box<Self>, message: &Message) -> Box<dyn MessageHandler> {
        self
    }

    fn has_quit(&self) -> bool {
        true
    }
}

fn handle_fatal_error(nick: &Nick, user_connections: &mut MutexGuard<'_, UserConnections>, err: ErrorType) -> Box<dyn MessageHandler> {
    error!("{err}");
    user_connections.remove_user(nick);
    Box::new(QuitHandler)
}
