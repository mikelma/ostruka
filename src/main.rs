use tokio;
use tokio::sync::Mutex;
use tokio::stream::StreamExt;
use tokio::time::delay_for;

use ostrich_core::{Command};
use ostruka_core::{Client, Message};

use std::net::SocketAddr;
use std::sync::Arc;
use std::process;
use std::time::Duration;

use ostruka::{handle_user, instance};
use instance::{Instance, Page};

mod config;
use config::Config;

#[tokio::main]
async fn main() {
    
    // Read the config file and deserialize it into Config
    let config = Config::new("config.toml").unwrap();

    let addr: SocketAddr = config.server_address.parse().unwrap();

    // Create a client and log in
    let (mut client, tx) = Client::log_in(&config.user, 
                                          &config.password, 
                                          addr).await
        .unwrap_or_else(|err| {
            eprintln!("Unable to log in as {} in {}. Error: {}", config.user, addr, err);
            process::exit(1);
        });

    // Init pages
    let home_page = Page::new("Hello".to_string(), vec![
        "---------------------------------------".to_string(),
        "Welcome to ostruka the ostrich client!".to_string(),
        "--------------------------------------".to_string(),
        "".to_string(),
    ]);

    // Init Instance
    let instance = Arc::new(Mutex::new(Instance::new()));
    let instance_clone = Arc::clone(&instance);

    // Add pages to instance
    instance.lock().await.add(home_page).unwrap();

    // Set Homepage as current Page
    instance.lock().await.set_current(0).unwrap();
    
    // Launch the user handler
    tokio::spawn(async move {
        if let Err(err) = handle_user(&config.user, tx, instance_clone).await {
            panic!("Fatal error: {}", err);
        }
    });

    while let Some(Ok(mesg)) = client.next().await {
        match mesg {
            Message::ToSend(cmd) => {
                if let Err(err) = client.send_cmd(&cmd).await {
                    instance.lock().await
                        .add_err(&format!("[✗] SERVER: {}", err))
                        .unwrap();
                }
            },
            Message::Received(Command::Msg(sender, _t, txt)) => {
                let result = instance.lock().await.add_msg(&sender, &txt);

                if let Err(err) = result {
                    instance.lock().await
                        .add_err(&err.to_string())
                        .unwrap()
                }
            },
            Message::Received(mesg) => {
                instance.lock().await
                    .add_line(None, &mesg.to_string())
                    .unwrap();
            },
        }
        // TODO : Is this needed? maybe to prevent DDOS?
        delay_for(Duration::from_millis(100)).await;
    }
}
