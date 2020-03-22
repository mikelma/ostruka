pub use ostruka_core::{Tx, Rx};

use tokio::sync::Mutex;

use crossterm::event::{self, Event, KeyCode};
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use std::io;
use std::process;
use std::time::Duration;
use std::sync::Arc;

pub mod commands;
pub mod instance;
mod ui;

use instance::Instance;

pub async fn handle_user(username: &str, mut tx: Tx, mut instance: Arc<Mutex<Instance>>) -> Result<(), io::Error> {
    // Set terminal into raw mode
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    // Init user's input buffer
    let mut user_buffer = String::new();

    loop {
        // Draw TUI
        if let Err(err) = ui::draw_tui(&mut terminal, 
                                       username,
                                       &user_buffer, 
                                       &instance).await {
                terminal.clear().unwrap();
                eprintln!("Fatal error, unable to draw TUI: {}", err);
                process::exit(1);
        }

        if let Some(special_key) = buffer_update(&mut user_buffer).unwrap() {
            // Process the special key
            match special_key {
                
                // The command is finished
                KeyCode::Enter => {
                    // Parse command
                    let command = commands::parse_command(&user_buffer);
                    // Run the command
                    if let Err(err) = commands::run_command(username, 
                                                            &command, 
                                                            &mut terminal,
                                                            &mut instance,
                                                            &mut tx).await {
                        
                        // Show error to the user
                        instance.lock().await
                            .add_err(&format!("[✗] ERR: {}", err))
                            .unwrap();
                    }
                    
                    // Clear user's buffer
                    user_buffer.clear();
                },

                KeyCode::Tab => instance.lock().await.next_page(),
                KeyCode::Down => instance.lock().await.scroll_down(),
                KeyCode::Up => instance.lock().await.scroll_up(),
                _ => (),
            }
        }
    }
}

pub fn buffer_update(buff: &mut String) -> crossterm::Result<Option<KeyCode>> {
    // `poll()` waits for an `Event` for a given time period
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(ev) => {
                match ev.code {
                    // If letter is given, add it to buffer
                    KeyCode::Char(c) => buff.push(c),

                    // If backspace, delete last word
                    KeyCode::Backspace => match buff.pop() {
                       Some(_c) => (),
                       None => (),
                    },
                    
                    // Else, return the special key pressed
                    _ => return Ok(Some(ev.code)),
                }
            },
            _ => (),
        }
    } else {
        // Timeout expired and no `Event` is available
    }
    Ok(None)
}    
