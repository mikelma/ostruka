use std::io;
use std::sync::Arc;

use tokio;
use tokio::sync::Mutex;

use tui::Terminal;
use tui::backend::Backend;
use tui::widgets::{Widget, Block, Borders, Text, Paragraph, Tabs};
use tui::layout::{Layout, Constraint, Direction, Alignment};
use tui::style::{Color, Style};

use crate::instance::Instance;

pub async fn draw_tui<B : Backend>(terminal: &mut Terminal<B>, 
                                   username: &str,
                                   user_buff: &str, 
                                   instance: &Arc<Mutex<Instance>>) -> Result<(), io::Error>{
    
    // Get the chat vector of the current page
    let chat = match instance.lock().await.get_chat() {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    // Get a vector with the names of all active pages
    let names = instance.lock().await.names();
    // Current pages index
    let current_index = instance.lock().await.get_current();

    // Create the layout, divide the screen
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
        .split(terminal.size()?);

    // Calculate the range of chat lines to be displayed in the text box
    // This method also controlls the scroll value
    let range = instance.lock()
        .await
        .display_range(chunks[1].height as usize);
    
    terminal.draw(move |mut f| {
        // Tabs
        Tabs::default()
            .block(Block::default().borders(Borders::ALL))
            .titles(names.as_slice())
            .style(Style::default().fg(Color::White))
            .select(current_index)
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider("|")
            .render(&mut f, chunks[0]);
        
        // Get the text vector from the chat of the current page
        let mut text = vec![];
        chat[range]
            .iter()
            .for_each(|txt| {
                text.push(Text::raw(format!("{}\n", txt)))
            });
        
        // Draw the text box (Paragraph)
        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .wrap(true)
            .render(&mut f, chunks[1]);

        // User input box, includes cursor and the alias of the user
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
