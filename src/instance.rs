use std::io;
use std::collections::HashMap;
use std::process;

pub struct Page{
    pub name : String,
    conversation : Vec<String>,
}

impl Page {

    /// Creates a new `Page` object. Fields to complete are the name of the 
    /// `Page`and the current conversation.
    pub fn new(name: String, conversation: Vec<String>) -> Page {
        Page { name, conversation}
    }
}

/// Holds a Krypto instance. It contains the instances's collection of `Pages`.
pub struct Instance{
    pages: Vec<Page>,
    current : usize, // Idex of the current `Page`.
}

impl Instance {

    /// Intitializes an empty `Instance` object.
    pub fn new() -> Instance {
        Instance { pages : Vec::new(), current : 0}
    }
    
    /// Intitializes `Instance` object from a vector of `Pages`.
    pub fn from(pages: Vec<Page>, current: usize) -> Instance {
        Instance { pages , current : current}
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

    pub fn add_line(&mut self, index: Option<usize>, message: String) -> Result<(), io::Error> {

        let index = match index {
            Some(i) => i,
            None => self.current,
        };
        
        // Check for invalid page indexes
        if index > self.pages.len() - 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, 
                               "The given index is not a valid index. Number too large."));
        }
        
        self.pages[index].conversation.push(message);

        Ok(())
    }
    
    // Adds a Command to the page of sender, if sender does not exist creates a new page
    pub fn add_msg(&mut self, sender: &str, txt: &str) -> Result<(), io::Error> {
        // Check if a page with the sender exists
        let index = self.names()
            .iter()
            .position(|name| *name == sender);

        let formatted = format!("[{}]: {}", sender, txt);

        match index {
            Some(i) => self.add_line(Some(i), formatted)?,
            None => {
                // Create a new page for the sender
                let page = Page::new(sender.to_string(), vec![formatted]);
                self.add(page)?;
            },
        }

        Ok(()) 
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
    
    /*
    pub fn update_recev(&mut self, client: &mut FtpClient) -> Result<(), io::Error> {
        let mut ls = client.update_recev()?;
        self.pages[self.current].conversation.append(&mut ls);
        Ok(())
    }
    */
}
