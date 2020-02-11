use std::io;
use std::sync::Arc;

use tokio;
use tokio::sync::Mutex;

use tui::Terminal;
use tui::backend::Backend;
use tui::widgets::{Widget, Block, Borders, Text, Paragraph, List, Tabs};
use tui::layout::{Layout, Constraint, Direction, Corner};
use tui::style::{Color, Style};

// use ostruka::{Instance};
use crate::instance::Instance;

// use crossterm::{execute, terminal};

pub async fn draw_tui<B : Backend>(terminal: &mut Terminal<B>, 
                         user_buff: &String, 
                         instance: &Arc<Mutex<Instance>>) -> Result<(), io::Error>{
    
    let chat = match instance.lock().await.get_chat() {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    let names = instance.lock().await.names();
    let current_index = instance.lock().await.get_current();

    terminal.draw(move |mut f| {

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(7),
                    Constraint::Percentage(82),
                    Constraint::Percentage(5)
                ].as_ref()
            )
            .split(f.size());
        

        // Tabs
        Tabs::default()
            .block(Block::default().borders(Borders::ALL))
            .titles(names.as_slice())
            .style(Style::default().fg(Color::White))
            .select(current_index)
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|")
            .render(&mut f, chunks[0]);
        
        // Chat
        let events = chat.iter()
            .map(|txt| {
                Text::raw(txt)
            });

        List::new(events)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .start_corner(Corner::TopLeft)
            .render(&mut f, chunks[1]);
        
        // User input box
        let text = [
            Text::styled("> ", Style::default().fg(Color::Red)),
            Text::raw(user_buff),
        ];

        Paragraph::new(text.iter())
                .block(Block::default().title("Input").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .wrap(true)
                .render(&mut f, chunks[2]);
    })
}
