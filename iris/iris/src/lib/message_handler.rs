//! # Connection State Storage
//! Stores various information about the connection
//! Implements state design pattern to handle transitions
//! See: https://refactoring.guru/design-patterns/state/rust/example

use crate::types::{Nick, Message, Reply, WelcomeReply, PrivMsg, PrivReply, ErrorType, ParsedMessage};
use crate::user_connections::UserConnections;
use crate::connect::ConnectionWrite;
use std::sync::Arc;
use log::{error};

/// The base connection state interface
pub trait MessageHandler {
    /// Handle an incoming message
    fn handle(self: Box<Self>, message: &Message) -> Box<dyn MessageHandler>;
}

/// A freshly started connection state
pub struct FreshHandler {
    user_connections: Arc<UserConnections>,
    curr_writer: ConnectionWrite,
}

impl FreshHandler {
    pub fn new(user_connections: Arc<UserConnections>, curr_writer: ConnectionWrite) -> Box<FreshHandler> {
        Box::new(FreshHandler { user_connections, curr_writer })
    }
}

impl MessageHandler for FreshHandler {
    fn handle(mut self: Box<Self>, message: &Message) -> Box<dyn MessageHandler> {
        match message {
            Message::Nick(nick_msg) => {
                let nick = nick_msg.nick.clone();
                self.user_connections.add_user(&nick, self.curr_writer);
                Box::new(NickedHandler { user_connections: self.user_connections, nick })
            },
            _ => self,
        }
    }
}

/// A connection state with a nick
pub struct NickedHandler {
    user_connections: Arc<UserConnections>,
    nick: Nick,
}

impl MessageHandler for NickedHandler {
    fn handle(self: Box<Self>, message: &Message) -> Box<dyn MessageHandler> {
        match message {
            Message::User(user_msg) => {
                let real_name = user_msg.real_name.clone();
                self.user_connections.write_to_user(
                    &self.nick, 
                    &Reply::Welcome(
                        WelcomeReply {
                            target_nick: self.nick.clone(),
                            message: format!("Hi {real_name}, welcome to IRC"),
                        }
                    )
                );

                Box::new(InitialisedHandler { 
                    user_connections: self.user_connections, 
                    nick: self.nick, 
                    real_name,
                })
            },
            _ => self,
        }
    }
}

/// Fully initialised connection
pub struct InitialisedHandler {
    user_connections: Arc<UserConnections>,
    nick: Nick,
    real_name: String,
}

impl MessageHandler for InitialisedHandler {
    fn handle(self: Box<Self>, message: &Message) -> Box<dyn MessageHandler> {
        self
    }
}
