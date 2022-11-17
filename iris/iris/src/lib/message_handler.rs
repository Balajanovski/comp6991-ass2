//! # Message handler
//! Implements state design pattern to handle transitions between handler states
//! Very loosely based off of: https://hoverbear.org/blog/rust-state-machine-pattern/

use crate::connect::{ConnectionError, ConnectionWrite};
use crate::types::{
    Message, Nick, ParsedMessage, PrivReply, QuitReply, Reply,
    WelcomeReply, JoinReply, PartReply,
};
use crate::user_connections::UserConnections;
use log::{error, info};
use std::sync::{Arc, Mutex};

pub enum HandlerState {
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
    state: HandlerState,
    user_connections: Arc<Mutex<UserConnections>>,
}

impl MessageHandler {
    pub fn new(
        user_connections: Arc<Mutex<UserConnections>>,
        curr_writer: ConnectionWrite,
    ) -> MessageHandler {
        MessageHandler {
            state: HandlerState::Fresh(Fresh {
                curr_writer: Arc::new(Mutex::new(curr_writer)),
            }),
            user_connections,
        }
    }

    pub fn handle(&mut self, message: anyhow::Result<String>) {
        match self.transition(message) {
            Ok(_) => {}
            Err(err) => {
                error!("{err}");

                if let Some(nick) = self.get_nick() {
                    let mut user_conn_guard = self.user_connections.lock().unwrap();
                    user_conn_guard.remove_user(&nick);
                }
            }
        }
    }

    pub fn has_quit(&self) -> bool {
        let has_quit = match self.state {
            HandlerState::Quit => true,
            _ => false,
        };

        has_quit
    }

    fn transition(&mut self, message: anyhow::Result<String>) -> anyhow::Result<()> {
        let message = message.as_deref().map(ParsedMessage::try_from);
        let message = match message {
            Ok(Ok(message)) => message,
            Err(err) => {
                match err.downcast_ref::<ConnectionError>() {
                    Some(ConnectionError::ConnectionLost | ConnectionError::ConnectionClosed) => {
                        info!("Lost connection.");

                        if let Some(nick) = self.get_nick() {
                            let mut user_conn_guard = self.user_connections.lock().unwrap();
                            user_conn_guard.remove_user(&nick);
                        }

                        self.state = HandlerState::Quit;
                        return Ok(());
                    },
                    Some(_) | None => {
                        error!("Invalid message received... ignoring message.");
        
                        return Ok(());
                    }
                }
            }
            Ok(Err(err)) => {
                error!("{err}");

                return Ok(());
            }
        };

        self.transition_parsed(message.message)
    }

    fn transition_parsed(&mut self, message: Message) -> anyhow::Result<()> {
        match (&self.state, message) {
            (HandlerState::Fresh(state), Message::Nick(nick_msg)) => {
                let nick = nick_msg.nick.clone();
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                user_conn_guard.add_user(&nick, state.curr_writer.clone())?;

                self.state = HandlerState::Nicked(Nicked { nick });
            }
            (HandlerState::Nicked(state), Message::User(user_msg)) => {
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

                self.state = HandlerState::Initialised(Initialised { nick, real_name });
            }
            (HandlerState::Initialised(state), Message::Ping(ping_msg)) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                let nick = state.nick.clone();
                user_conn_guard.write_to_user(&nick, &Reply::Pong(ping_msg.clone()))?;
            }
            (HandlerState::Initialised(state), Message::Quit(quit_msg)) => {
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

                self.state = HandlerState::Quit;
            }
            (HandlerState::Initialised(state), Message::PrivMsg(priv_msg)) => {
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
            (HandlerState::Initialised(state), Message::Join(join_msg)) => {
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
            (HandlerState::Initialised(state), Message::Part(part_msg)) => {
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
            _ => { },
        };

        Ok(())
    }

    fn get_nick(&self) -> Option<Nick> {
        let nick = match &self.state {
            HandlerState::Nicked(state) => Some(state.nick.clone()),
            HandlerState::Initialised(state) => Some(state.nick.clone()),
            _ => None,
        };

        nick
    }
}

/*

/// Fully initialised connection
pub struct InitialisedHandler {
    user_connections: Arc<Mutex<UserConnections>>,
    nick: Nick,
    real_name: String,
}

impl MessageHandler for InitialisedHandler {
    fn handle_parsed(self: Box<Self>, message: &Message) -> Result<Box<dyn MessageHandler>, ErrorType> {
        match message {
            Message::Ping(ping_msg) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                user_conn_guard.write_to_user(&self.nick, &Reply::Pong(ping_msg.clone()))?;
                Ok(self)
            }
            Message::Quit(quit_msg) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                user_conn_guard.write_to_users_channel(
                    &self.nick,
                    &Reply::Quit(QuitReply {
                        message: quit_msg.clone(),
                        sender_nick: self.nick.clone(),
                    }),
                )?;

                user_conn_guard.remove_user(&self.nick);
                Ok(Box::new(QuitHandler))
            }
            Message::PrivMsg(priv_msg) => {
                let mut user_conn_guard = self.user_connections.lock().unwrap();
                user_conn_guard.write(
                    &priv_msg.target,
                    &Reply::PrivMsg(PrivReply {
                        message: priv_msg.clone(),
                        sender_nick: self.nick.clone(),
                    }),
                )?;

                Ok(self)
            }
            _ => Ok(self),
        }
    }

    fn handle_fatal_error(&self, err: ErrorType) -> Box<dyn MessageHandler> {
        let mut user_conn_guard = self.user_connections.lock().unwrap();
        error!("{err}");
        user_conn_guard.remove_user(&self.nick);
        Box::new(QuitHandler)
    }
}

/// The session has quit
pub struct QuitHandler;

impl MessageHandler for QuitHandler {
    fn handle_parsed(self: Box<Self>, message: &Message) -> Result<Box<dyn MessageHandler>, ErrorType> {
        Ok(self)
    }

    fn handle_fatal_error(&self, err: ErrorType) -> Box<dyn MessageHandler> {
        Box::new(QuitHandler)
    }

    fn has_quit(&self) -> bool {
        true
    }
}
*/
