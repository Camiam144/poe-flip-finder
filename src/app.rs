use std::io::{self, Error, Write};

use reqwest::blocking::Client;

pub struct App {
    should_quit: bool,
    min_volume: f64,
    min_profit_frac: f64,
    client: Option<Client>,
}

impl App {
    pub const fn default() -> Self {
        Self {
            should_quit: false,
            min_volume: 1000.0,
            min_profit_frac: 0.05,
            client: None,
        }
    }

    pub fn run(&mut self) {
        self.initialize();
        let result = self.repl();
    }

    pub fn repl(&mut self) -> Result<(), Error> {
        loop {
            // I'm using built in libraries for now, this could always
            // be grabbed from a different crate later.
            if self.should_quit {
                break;
            }
            print!(">> ");
            io::stdout().flush().unwrap();

            let mut line = String::new();
            if io::stdin().read_line(&mut line).is_err() {
                println!("Error reading input, try again.");
                continue;
            }
            let cmd = line.trim();
            if cmd.is_empty() {
                continue;
            }
            if cmd == "quit" || cmd == "exit" {
                self.should_quit = true;
            }
        }
        Ok(())
    }
    pub fn set_volume(&mut self, vol: f64) {
        self.min_volume = vol;
    }

    pub fn initialize(&mut self) {
        println!("Welcome to POE 2 FLIP FINDER!");
        println!("Running initial setup...");
        let client = Some(self.build_client().expect("Couldn't create client: "));
        self.client = client
    }

    pub fn build_client(&self) -> Result<reqwest::blocking::Client, reqwest::Error> {
        reqwest::blocking::Client::builder()
            .user_agent("poe-flip-finder/1.0-camiam144@gmail.com")
            .build()
    }
}
