use std::io;
use std::sync::Arc;

use tokio;
use tokio::sync::Mutex;

use tui::Terminal;
use tui::backend::Backend;
use tui::widgets::{Widget, Block, Borders, Text, Paragraph, List, SelectableList};
use tui::layout::{Layout, Constraint, Direction, Alignment};
use tui::style::{Color, Style, Modifier};

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
    // Get currently online users, if the current page is not a group chat None is returned
    let online_users = instance.lock().await.get_online_users();

    // Create the vertical layout, divide the screen into two sections
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Percentage(95),
            Constraint::Percentage(5)
        ].as_ref())
        .split(terminal.size()?);

    // Divide the firts vertical block into three horizontal brlocks 
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Min(15),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ].as_ref())
        .split(v_chunks[0]);

    // Calculate the range of chat lines to be displayed in the text box
    // This method also controlls the scroll value
    let range = instance.lock()
        .await
        .display_range(h_chunks[1].height as usize);
    
    terminal.draw(move |mut f| {
        // Tabs
        SelectableList::default()
            .block(Block::default().borders(Borders::RIGHT))
            .items(&names)
            .select(Some(current_index))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default()
                .fg(Color::Magenta)
                .modifier(Modifier::UNDERLINED))
            .highlight_symbol(">")
            .render(&mut f, h_chunks[0]);

        if let Some(online_list) = online_users {
            let txt = online_list.iter().map(|a| Text::raw(a.as_str()));
            List::new(txt)
                .block(Block::default().borders(Borders::LEFT))
                .style(Style::default().fg(Color::White))
                .render(&mut f, h_chunks[2]);
        }
        
        // Get the text vector from the chat of the current page
        let mut text = vec![];
        chat[range]
            .iter()
            .for_each(|txt| {
                text.push(Text::raw(format!("{}\n", txt)))
            });
        
        // Draw the text box (Paragraph)
        Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .wrap(true)
            .render(&mut f, h_chunks[1]);

        // User input box, includes cursor and the alias of the user
        let text = [
            Text::styled(format!("({})> ", username), 
                         Style::default()
                         .fg(Color::LightMagenta)
                         .modifier(Modifier::BOLD)),
            Text::raw(format!("{}▌", user_buff)),
        ];
        Paragraph::new(text.iter())
                .block(Block::default().borders(Borders::TOP))
                .style(Style::default().fg(Color::White))
                .wrap(true)
                .render(&mut f, v_chunks[1]);
    })
}
