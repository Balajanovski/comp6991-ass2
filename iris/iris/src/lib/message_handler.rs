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
    fn handle(self: Box<Self>, message: &str) -> Box<dyn MessageHandler>;
}

/// A freshly started connection state
pub struct FreshHandler {
    user_connections: Arc<UserConnections>,
    curr_writer: Box<ConnectionWrite>,
}

impl FreshHandler {
    pub fn new(user_connections: Arc<UserConnections>, curr_writer: Box<ConnectionWrite>) -> Box<FreshHandler> {
        Box::new(FreshHandler { user_connections, curr_writer })
    }
}

impl MessageHandler for FreshHandler {
    fn handle(mut self: Box<Self>, message: &str) -> Box<dyn MessageHandler> {
        let message = ParsedMessage::try_from(message);
        let message = match message {
            Ok(message) => message,
            Err(err) => { 
                error!("{err}");
                return self;
            },
        };

        self
    }
}
