use std::io::{self, Error, Write};

use reqwest::blocking::Client;

pub struct App {
    should_quit: bool,
    min_volume: f64,
    min_profit_frac: f64,
    top: usize,
    client: Option<Client>,
}

impl App {
    pub const fn default() -> Self {
        Self {
            should_quit: false,
            min_volume: 1000.0,
            min_profit_frac: 0.05,
            top: 10,
            client: None,
        }
    }

    pub fn initialize(&mut self) {
        println!("Welcome to POE 2 FLIP FINDER!");
        println!("Running initial setup...");
        let client = Some(self.build_client().expect("Couldn't create client: "));
        self.client = client
    }

    pub fn run(&mut self) {
        self.initialize();
        let result = self.repl();
        result.unwrap()
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
            self.handle_command(cmd);
        }
        Ok(())
    }

    // pub fn set_volume(&mut self, vol: f64) {
    //     self.min_volume = vol;
    // }

    pub fn build_client(&self) -> Result<reqwest::blocking::Client, reqwest::Error> {
        reqwest::blocking::Client::builder()
            .user_agent("poe-flip-finder/1.0-camiam144@gmail.com")
            .build()
    }

    pub fn handle_command(&mut self, cmd: &str) {
        // right now, valid commands are:
        // volume #
        // profit #.#
        // top #
        // quit | exit
        let mut cmd_parts: std::str::SplitWhitespace<'_> = cmd.split_whitespace();

        match cmd_parts.next() {
            Some("volume") => {
                if let Some(arg) = cmd_parts.next() {
                    if let Ok(vol) = arg.parse::<f64>() {
                        self.min_volume = vol;
                        println!("Min volume set {vol}");
                    } else {
                        println!("Invalid setting for volume, use an integer.");
                    }
                }
            }
            Some("profit") => {
                if let Some(arg) = cmd_parts.next() {
                    if let Ok(profit) = arg.parse::<f64>() {
                        self.min_profit_frac = profit;
                        println!("Min profit frac set to {profit}");
                    } else {
                        println!("Invalid setting for profit.");
                    }
                }
            }
            Some("top") => {
                if let Some(arg) = cmd_parts.next() {
                    if let Ok(top) = arg.parse::<usize>() {
                        self.top = top;
                        println!("Displaying top {top} results.");
                    } else {
                        println!("Invalid value for top, use int.")
                    }
                }
            }
            Some("quit") | Some("exit") => self.should_quit = true,
            Some(other) => println!("Unknown command: {other}"),
            None => {}
        };
    }
}
