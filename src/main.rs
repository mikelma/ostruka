use tokio;
use tokio::sync::Mutex;
use tokio::stream::StreamExt;
//use tokio::time::delay_for;

use ostrich_core::*;
use ostruka_core::{Client, Message};

use std::net::SocketAddr;
use std::sync::Arc;
use std::process;
//use std::time::Duration;

use ostruka::{handle_user, instance};
use instance::{Instance, Page};

mod config;
use config::Config;

#[tokio::main]
async fn main() {
    
    // Read the config file and deserialize it into Config
    let config = Config::new("config.toml").unwrap();

    let addr: SocketAddr = config.server_address.parse().unwrap();

    println!("Trying to log in as {} in {}...", 
             config.user, config.server_address);
    
    // The name variable is initialized in this point as its no clear wich of the username options
    // is going to be chosen before trying to log in.
    let user_alias: String;

    // Create a client and try to log in
    let (mut client, tx) = match Client::log_in(&config.user, 
                                                &config.password, 
                                                addr).await {
        Ok(result) => {
            user_alias = config.user; // The first alias is selected
            result
        },
        Err(err) => {
            // The first try was unsuccessful, try with the second option
            eprintln!("Unable to log in as {} in {}. Error: {}", 
                      config.user, addr, err);
            println!("Trying to log in as {}", config.user_option_2);
            
            // Second try
            match Client::log_in(&config.user_option_2,
                                 &config.password,
                                 addr).await {
                Ok(result) => {
                    user_alias = config.user_option_2; // The second alias is selected
                    result
                },
                Err(err) => {
                    // Failed to log in again, time to exit
                    eprintln!("Unable to log in as {} in {}. Error: {}", 
                              config.user_option_2, addr, err);
                    eprintln!("Cannot connect to server. Exiting...");
                    process::exit(1);
                },
            }
        },
    };

    // Init pages
    let home_page = Page::new("ostruka".to_string(), vec![
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
        if let Err(err) = handle_user(&user_alias, tx, instance_clone).await {
            panic!("Fatal error: {}", err);
        }
    });

    while let Some(Ok(mesg)) = client.next().await {
        match mesg {
            // Send a command to the server
            Message::ToSend(cmd) => {
                if let Err(err) = client.send_cmd(&cmd).await {
                    instance.lock().await
                        .add_err(&format!("[✗] SERVER: {}", err))
                        .unwrap();
                }
            },
            // Received a MSG command from server. Locate the message 
            // in its correspondig page
            Message::Received(Command::Msg(sender, target, txt)) => {
                let result = instance.lock().await.add_msg(&sender, &target, &txt);

                if let Err(err) = result {
                    instance.lock().await
                        .add_err(&err.to_string())
                        .unwrap()
                }
            },
            Message::Received(Command::ListUsr(group, op, users)) => {
                // Users are sent by the server in a single string divided by newline
                let splitted = users.split('\n');
                // let list: Vec<String> = splitted.map(|s| s.trim().to_string()).collect();
                
                let mut list: Vec<String> = vec![];
                // NOTE: This if len > 0 condition is needed as the splitted vector might 
                // contain some empty items sometimes.
                splitted.for_each(|s| if s.len() > 0 { list.push(s.to_string()) });
                match op {
                    ListUsrOperation::Add => instance.lock().await.add_online_users(&group, list),
                    ListUsrOperation::Remove => instance.lock().await.remove_online_users(&group, list),
                }
            },
            // Received messages from the server that are not messages are treated here
            Message::Received(mesg) => {
                instance.lock().await
                    .add_line(None, &mesg.to_string())
                    .unwrap();
            },
        }
    }
}
