use std::io;
// use std::collections::HashMap;
use std::process;
use std::ops::Range;

pub struct Page{
    pub name : String,
    conversation : Vec<String>,
    pub scroll: usize,
}

impl Page {

    /// Creates a new `Page` object. Fields to complete are the name of the 
    /// `Page`and the current conversation.
    pub fn new(name: String, conversation: Vec<String>) -> Page {
        Page { name, conversation, scroll: 0 }
    }
}

/// Holds a Krypto instance. It contains the instances's collection of `Pages`.
pub struct Instance{
    pages: Vec<Page>,
    current : usize, // Idex of the current `Page`.
    pub screen_len: Option<usize>,
}

impl Instance {

    /// Intitializes an empty `Instance` object.
    pub fn new() -> Instance {
        Instance { pages : Vec::new(), current : 0, screen_len: None}
    }
    
    /// Intitializes `Instance` object from a vector of `Pages`.
    pub fn from(pages: Vec<Page>, current: usize) -> Instance {
        Instance { pages , current : current, screen_len: None }
    }
    
    /// Adds a `Page` to the current `Instance`. If the page is already added, 
    /// returns an io::Error.
    pub fn add(&mut self, page: Page) -> Result<(), io::Error> {

        // Check if the user is already talking with the user that want's to join
        if self.names().contains(&page.name) {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, 
                                      format!("Already joined {}", page.name)))
        }
        
        // Add the new page to the instance
        self.pages.push(page); 

        Ok(())
    }

    /// Returns a vector of names of the current `Page`s inside the `Instance`.
    pub fn names(&self) -> Vec<String> {

        if self.pages.len() > 0 {
            let mut names_list = Vec::new();
            self.pages.iter()
                .for_each(|page| names_list.push(page.name.clone()));

            names_list
            
        } else {
            vec!["".to_string()]
        }
    }
    
    /// Returns a reference to a vector of Strings containing the messages of the current chat.
    pub fn get_chat(&self) -> Result<Box<Vec<String>>, io::Error> {
        let txt = self.pages[self.current]
            .conversation
            .clone();

        Ok(Box::new(txt))
    }
    
    /// Returns the name of the curent page
    pub fn get_name(&self) -> String {
        self.pages[self.current].name.clone()
    }

    /// Changes the name of the curent page to the given name
    pub fn set_name(&mut self, new_name: String) {
        self.pages[self.current].name = new_name;
    }
    
    /// Sets the current `Page` to the page of the given index.
    pub fn set_current(&mut self, index: usize) -> Result<(), io::Error> {
        if index > self.pages.len() - 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, 
                               "The given index is not a valid index. Number too large."));
        }
        self.current = index;
        Ok(())
    }
    
    // Returns the index of the current `Page`.
    pub fn get_current(&self) -> usize {
        self.current
    }
    
    /// Change the current page selection to the next tabe.
    /// If the current page is the last, select the first page as next.
    pub fn next_page(&mut self) {
        if self.pages.len() > 1 {
            if self.current == self.pages.len() - 1 {
                self.current = 0;
            } else {
                self.current += 1;
            }
        }
    } 

    pub fn add_line(&mut self, index: Option<usize>, message: &str) -> Result<(), io::Error> {

        let index = match index {
            Some(i) => i,
            None => self.current,
        };
        
        // Check for invalid page indexes
        if index > self.pages.len() - 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, 
                               "The given index is not a valid index. Number too large."));
        }
        
        self.pages[index].conversation.push(message.to_string());

        Ok(())
    }
    
    /// Adds a Command to the page of sender, if sender does not exist creates a new page.
    /// If target's name starts with '#', recognize the message as a group message.
    pub fn add_msg(&mut self, sender: &str, target: &str, txt: &str) -> Result<(), io::Error> {
        // Determine if the message is a group message or not
        let name = if target.starts_with("#") {
            target   
        } else {
            sender
        };

        // Check if a page with the sender exists
        let index = self.names()
            .iter()
            .position(|page_name| *page_name == name);

        let formatted = format!("[{}]: {}", sender, txt);

        match index {
            Some(i) => self.add_line(Some(i), &formatted)?,
            None => {
                // Create a new page for the sender
                let page = Page::new(sender.to_string(), vec![formatted]);
                self.add(page)?;
            },
        }

        Ok(()) 
    }

    /// Adds an error description to the current page
    pub fn add_err(&mut self, error: &str) -> Result<(), io::Error> {
        self.add_line(None, &format!("[ERR]: {}", error))
    }

    pub fn add_chat(&mut self, index: usize, message: &mut Vec<String>) -> Result<(), io::Error> {
        
        // Check for invalid page indexes
        if index > self.pages.len() - 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, 
                               "The given index is not a valid index. Number too large."));
        }
        
        self.pages[index].conversation.append(message);

        Ok(())
    }
    
    /*
    pub fn update(&mut self, update: &mut HashMap<String, Vec<String>>) {

        for key in update.keys() {
            let sender = match key.to_string().split('@').nth(0){
                Some(s) => s.to_string(),
                None => continue, // NOTE, duda ondo dagoen
            };

            let mut data = match update.get(key) {
                Some(d) => d.clone(), // TODO: Change clone, get_mut()
                None => continue,
            };
            
            match self.names().binary_search(&sender) {
                Ok(index) => self.add_chat(index, &mut data).unwrap(),
                Err(_) => continue, // TODO Create a new page !!
            }
        }
    }
    */
    
    /// Removes the current page and jumps to the next page.
    pub fn remove_current(&mut self) -> Result<(), io::Error> {

        if self.pages.len() == 1 {
            // If the user is in the home directory, quit
            process::exit(0);
        }

        let index = self.current;

        self.next_page();
        
        self.pages.remove(index);
        Ok(())
    }
        
    /// Scroll down the current page
    pub fn scroll_down(&mut self) {
        // Scroll always positive
        if self.pages[self.current].scroll > 0 {
            self.pages[self.current].scroll -= 1;
        }
    }

    /// Scroll up the current page
    pub fn scroll_up(&mut self) {
        self.pages[self.current].scroll += 1;
    }
    
    /// Returns the correct range of chat lines to be displayed (lines of the current page).
    /// Also controlls the scroll value range before calculating the range.
    pub fn display_range(&mut self, screen_len: usize) -> Range<usize> {
        // Get values
        let chat_len = self.pages[self.current].conversation.len();
        let scroll = self.get_scroll();

        // control scroll value bounds
        if chat_len + 2 < screen_len {
            self.pages[self.current].scroll = 0;
        } else {
            if scroll > chat_len + 2 - screen_len {
                self.pages[self.current].scroll = chat_len + 2 - screen_len;
            }
        }
        // Get new scroll value 
        let scroll = self.get_scroll();
        
        // Select the correct range of chat lines to display
        // It is not allowed to scroll if the chat fits in the screen height
        if chat_len + 2 >= screen_len && chat_len+2-screen_len >= scroll {
            (chat_len+2-screen_len-scroll..chat_len-scroll)
        } else {
            (0..chat_len)
        }
    }

    fn get_scroll(&self) -> usize {
       self.pages[self.current].scroll 
    }
    
    /// Sets the scroll value of the current page to zero
    pub fn scroll_zero(&mut self) {
       self.pages[self.current].scroll = 0;
    }
     
}
