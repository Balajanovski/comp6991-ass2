#[macro_use]
extern crate log;
extern crate simplelog;

use clap::Parser;
use iris_lib::{
    connect::{ConnectionManager},
    message_handler::MessageHandler,
    types::{SERVER_NAME},
    user_connections::UserConnections,
};
use simplelog::*;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::anyhow;

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
        SERVER_NAME, arguments.ip_address, arguments.port
    );

    let mut connection_manager = ConnectionManager::launch(arguments.ip_address, arguments.port);
    let user_connections = Arc::new(Mutex::new(UserConnections::new()));

    thread::scope(|s| {
        loop {
            // This function call will block until a new client connects!
            let (mut conn_read, conn_write) = connection_manager.accept_new_connection();
            let thread_user_connections = user_connections.clone();
            info!("New connection from {}", conn_read.id());

            s.spawn(move || {
                let mut handler = MessageHandler::new(thread_user_connections, conn_write);
                while !handler.has_quit() {
                    info!("Waiting for message...");

                    let message = conn_read.read_message();
                    handler.handle(message.map_err(|e| anyhow!(e)));
                }

                info!("Connection has closed...");
            });
        }
    })
}
