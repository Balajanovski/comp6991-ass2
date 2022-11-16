#[macro_use] extern crate log;
extern crate simplelog;

use clap::Parser;
use iris_lib::{
    connect::{ConnectionError, ConnectionManager},
    types::{SERVER_NAME, ParsedMessage, Message, Reply},
    message_handler::{FreshHandler, MessageHandler},
    user_connections::UserConnections,
};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use simplelog::*;

#[derive(Parser)]
struct Arguments {
    #[clap(default_value = "127.0.0.1")]
    ip_address: IpAddr,

    #[clap(default_value = "6991")]
    port: u16,
}

fn main() {
    let arguments = Arguments::parse();
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    info!(
        "Launching {} at {}:{}",
        SERVER_NAME,
        arguments.ip_address,
        arguments.port
    );

    let mut connection_manager = ConnectionManager::launch(arguments.ip_address, arguments.port);
    let user_connections = Arc::new(Mutex::new(UserConnections::new()));

    loop {
        // This function call will block until a new client connects!
        let (mut conn_read, conn_write) = connection_manager.accept_new_connection();

        info!("New connection from {}", conn_read.id());

        let mut handler: Box<dyn MessageHandler> = FreshHandler::new(user_connections.clone(), conn_write);
        while !MessageHandler::has_quit(handler.as_ref()) {
            info!("Waiting for message...");

            let message = conn_read.read_message();
            let message = message.as_deref().map(ParsedMessage::try_from);

            /// TODO: Deal with deleting from UserConnections on an error here happening
            let message = match message {
                Ok(Ok(message)) => message,
                Err(ConnectionError::ConnectionLost | ConnectionError::ConnectionClosed) => {
                    info!("Lost connection.");
                    break;
                },
                Err(_) => {
                    error!("Invalid message received... ignoring message.");
                    continue;
                },
                Ok(Err(err)) => {
                    error!("{err}");
                    continue;
                },
            };

            handler = MessageHandler::handle(handler, &message.message);
        }

        info!("Connection has closed...");
    }
}
