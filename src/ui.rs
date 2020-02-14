use std::io;
use std::sync::Arc;

use tokio;
use tokio::sync::Mutex;

use tui::Terminal;
use tui::backend::Backend;
use tui::widgets::{Widget, Block, Borders, Text, Paragraph, Tabs};
use tui::layout::{Layout, Constraint, Direction, Alignment};
use tui::style::{Color, Style};

// use ostruka::{Instance};
use crate::instance::Instance;

// use crossterm::{execute, terminal};

pub async fn draw_tui<B : Backend>(terminal: &mut Terminal<B>, 
                                   username: &str,
                                   user_buff: &str, 
                                   instance: &Arc<Mutex<Instance>>) -> Result<(), io::Error>{

    let percentages: Vec<u16> = vec![7, 82, 5];

    let chat = match instance.lock().await.get_chat() {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    let names = instance.lock().await.names();
    let current_index = instance.lock().await.get_current();

    let scroll = instance.lock().await.get_scroll();
    
    terminal.draw(move |mut f| {

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(percentages[0]),
                    Constraint::Percentage(percentages[1]),
                    Constraint::Percentage(percentages[2])
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
        let skip = if chat.len() + 2 >= chunks[1].height as usize {
            chat.len() + 2 - (chunks[1].height as usize)
        } else {
           0  
        };

        let mut text = vec![];
        chat.iter()
            .skip(skip)
            .for_each(|txt| {
                text.push(Text::raw(format!("{}\n", txt)))
            });

        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .wrap(true)
            .scroll(scroll)
            .render(&mut f, chunks[1]);

        // User input box, include cursor
        let text = [
            Text::styled(format!("({})> ", username), 
                         Style::default().fg(Color::LightMagenta)),
            Text::raw(format!("{}▌", user_buff)),
        ];

        Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .wrap(true)
                .render(&mut f, chunks[2]);
    })
}
