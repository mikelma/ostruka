use std::io;
use std::process;
use std::sync::Arc;

use tokio;
use tokio::sync::Mutex;

use tui::Terminal;
use tui::backend::Backend;

extern crate sha1;
// use sha1::*;

use crate::instance::{Instance, Page};

use ostrich_core::Command;

use crate::Tx;

#[derive(Debug)]
pub enum UserCommand {
    Message(String),    // Contains a message to send
    ChangePage(usize),  // Change the current page to the specified
    Join(String),       // Conatins the name of the new page
    ScrollUp,
    ScrollDown,
    Close,              // Close the current page
    Exit,               // exit ostruka
    Unknown(String),
}

pub fn parse_command(input: &str) -> UserCommand {
    // let input = input.to_lowercase(); 

    // Exit command
    if input.starts_with(":exit") {
        UserCommand::Exit
    
    // Join command
    } else if input.starts_with(":join") {

        let mut join_name = input.to_string();
        let _= join_name.drain(..6); // 6, includes whitespace

        UserCommand::Join(join_name)

    // Close command
    } else if input == ":q" || input == ":close" {
        UserCommand::Close

    } else if input.starts_with(':') {

        let no_start = input.trim_matches(':');
        
        // ChangePage commad
        if let Ok(num) = no_start.parse() {
            return UserCommand::ChangePage(num);
        }

        // Unknown command
        UserCommand::Unknown(input.to_string())

    // Message command 
    } else {
        UserCommand::Message(input.to_string())
    }
} 

pub async fn run_command<B: Backend>(username: &str,
                                     command: &UserCommand,
                                     terminal: &mut Terminal<B>,
                                     instance: &Arc<Mutex<Instance>>,
                                     client_tx: &mut Tx) -> Result<(), io::Error> {

    match command {
        
        // Exit: exit the program
        UserCommand::Exit => {
            terminal.show_cursor()?;
            terminal.clear().unwrap();
            process::exit(0)
        }, 
        
        UserCommand::Message(ms) => {
            // Get the target's name from the name of the current page
            let target = instance.lock().await
                .get_name();

            // If the target is ostruka the user is in the main page,
            // so ignore the send command and just print the message.
            // Also ignore sending command if the message's length is 0.
            if target != "ostruka" && !ms.is_empty() {
                let command = Command::Msg(username.to_string(),
                                           target,
                                           ms.to_string());

                // Create a Command to send to the client
                if let Err(_) = client_tx.send(command) {
                    return Err(io::Error::new(io::ErrorKind::ConnectionAborted, 
                                              "Client died, nothing to do"));
                }
                // Display message in the Page
                instance.lock().await
                    .add_line(None, &format!("({})> {}", username, ms))?;
            }
        },

        // Try to change the page. If cannot change, display not valid index;
        UserCommand::ChangePage(new_index) => {
            let result = instance.lock().await.set_current(*new_index);
            if let Err(err) = result {
                instance.lock().await
                        .add_err(&err.to_string())?; 
            }
        },

        UserCommand::Join(name) => {
            // Send a Join command
            if let Err(e) = client_tx.send(Command::Join(name.clone())) {
                // Display the error in the current page
                instance.lock().await.add_err(&format!("Join command error: {}", e)).unwrap();
            } else {
                // If send successful, add a new page
                instance.lock().await.add(
                    Page::new(name.clone(), vec![format!("Joined {}!", name)])
                )?;
            }

        },

        UserCommand::Close => {
            instance.lock().await.remove_current()?;
        },
        
        // Display a warning in the current page about the unknown command
        UserCommand::Unknown(c) => {
            let _ = instance.lock().await
                .add_err(&format!("{}: Unknown command", c))?;
        },
        
        // For ScrollDown and ScrollUp, that cannot be runned in this function
        _ => {
            let _ = instance.lock().await
                .add_err("Cannot run command")?;
        },
    }

    Ok(())
}
