use serde::Deserialize;
use toml;

use std::fs::File;
use std::io::{self, Read};

#[derive(Deserialize)]
pub struct Config {
    pub user: String,
    pub user_option_2: String,

    pub password: String,

    pub server_address: String,
}

impl Config {
    
    pub fn new(config_path: &str) -> Result<Config, io::Error> {
        // Read configuration file
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Deserialize the config string
        Ok(toml::from_str(&contents)
           .expect("Unable to deserialize the config file"))
    }
}
