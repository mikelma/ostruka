use std::io;
use std::ops::Range;

pub struct Page {
    pub name : String,
    conversation : Vec<String>,
    pub scroll: usize,
    pub n_lines: usize,
    online_users: Option<Vec<String>>,
}

impl Page {

    /// Creates a new `Page` object. Fields to complete are the name of the 
    /// `Page`and the current conversation.
    pub fn new(name: String, conversation: Vec<String>) -> Page {
        let n_lines = conversation.len();
        Page { name, conversation, scroll: 0, n_lines, online_users: None}
    }
}

/// Holds an ostruka instance. It contains the instances's collection of `Pages`.
pub struct Instance {
    pages: Vec<Page>, // NOTE: Upgrade to hashmap
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
        Instance { pages, current, screen_len: None }
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
        
        if message.contains('\n') {
            for line in message.split('\n') {
                self.pages[index].conversation.push(line.to_string());
                self.pages[index].n_lines += 1;
            }
        } else {
            self.pages[index].conversation.push(message.to_string());
            self.pages[index].n_lines += 1;
        }

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
    
    /// Removes the current page and jumps to the next page.
    pub fn remove_current(&mut self) -> Result<(), io::Error> {
        // The user is in the main page and has more pages active
        if self.current == 0 {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                    "Cannot quit main page, run :exit to quit ostruka"))
        }

        // The user wants to remove an active page
        let index = self.current; // Save the index
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
        let chat_len = self.pages[self.current].n_lines;
        let scroll = self.get_scroll();

        // control scroll value bounds
        if chat_len + 2 < screen_len {
            self.pages[self.current].scroll = 0;

        } else if scroll > chat_len + 2 - screen_len {
            self.pages[self.current].scroll = chat_len + 2 - screen_len;
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
    
    /// Adds a given vector of new usernames to the online_users field of the `Page` of 
    /// the specified name. If the given `Page` name does not exist, nothing is done.
    pub fn add_online_users(&mut self, page_name: &str, new_users: Vec<String>) {
        if let Some(index) = self.pages.iter_mut()
            .position(|page| page.name == page_name) {

            if let Some(list) = &mut self.pages[index].online_users {
                new_users.iter().for_each(|x| list.push(x.clone()));
            } else {
                self.pages[index].online_users = Some(new_users);
            }
        }
    }
    
    /// Removes usernames in the online_users field of the `Page` specified determined by 
    /// a vector of given usernames. If a username of the given vector does not exist in the
    /// online_users of the selected `Page`, the username to remove is ignored. Also,
    /// if the given `Page` name does not exist, nothing is done.
    pub fn remove_online_users(&mut self, page_name: &str, users: Vec<String>) {
        if let Some(index) = self.pages.iter_mut() 
            .position(|page| page.name == page_name) {

                if let Some(list) = &mut self.pages[index].online_users {
                    users.iter()
                        .for_each(|user| {
                            if let Some(i) = list.iter().position(|x| x == user) {
                                list.remove(i);
                            }
                        });
                }
            }
    }

    pub fn get_online_users(&self) -> Option<Vec<String>> {
        self.pages[self.current].online_users.clone()
    }
}
