use tokio;
use tokio::sync::Mutex;
use tokio::stream::StreamExt;

use ostrich_core::{RawMessage, Command};
use ostruka_core::Client;

use std::net::SocketAddr;
use std::process;
use std::io;
use std::sync::Arc;

mod instance;
use instance::{Page, Instance};

use ostruka::handle_user;

#[tokio::main]
async fn main() {
    // TODO: Read from config page 
    let username = "mike";
    let password = "9d4e1e23bd5b727046a9e3b4b7db57bd8d6ee684"; // TODO: Encrypt and sign
    let addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

    // Create a client and log in
    let (mut client, tx) = Client::log_in(username, password, addr).await.unwrap();

    // Init pages
    let home_page = Page::new("Hello".to_string(), vec![
        "--------------------------".to_string(),
        "Welcome to Ostrich client!".to_string(),
        "--------------------------".to_string(),
        "".to_string(),
    ]);

    // Init Instance
    let mut instance = Arc::new(Mutex::new(Instance::new()));

    // Add pages to instance
    instance.lock().await.add(home_page).unwrap();

    // Set Homepage as current Page
    instance.lock().await.set_current(0).unwrap();

    tokio::spawn(async move {
        if let Err(err) = handle_user() {
            panic!("Fatal error: {}", err);
        }
    });
}
