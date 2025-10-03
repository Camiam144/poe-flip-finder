use std::{
    fs,
    io::{self, Error, Write},
    path::PathBuf,
};

use reqwest::blocking::Client;

use crate::{
    api, logic,
    models::{
        api_models::ExchangeRecord,
        logic_models::{TradingCurrencyRates, TradingCurrencyType},
    },
};

pub struct App {
    should_quit: bool,
    min_volume: f64,
    min_profit_frac: f64,
    top: usize,
    client: Option<Client>,
    data_path: PathBuf,

    // Non-user facing data
    current_snapshot: Option<u64>,
    current_records: Option<Vec<ExchangeRecord>>,
    base_rates: TradingCurrencyRates,
    results: Option<Vec<(TradingCurrencyType, String, TradingCurrencyType, f64)>>,
}

impl App {
    pub fn default() -> Self {
        Self {
            should_quit: false,
            min_volume: 1000.0,
            min_profit_frac: 0.05,
            top: 10,
            client: None,
            data_path: PathBuf::from("data"),

            current_snapshot: None,
            current_records: None,
            base_rates: TradingCurrencyRates::default(),
            results: None,
        }
    }

    pub fn initialize(&mut self) {
        println!("Welcome to POE 2 FLIP FINDER!");
        println!("Running initial setup...");
        println!("Use help for valid commands");
        let client = Some(self.build_client().expect("Couldn't create client: "));
        self.client = client;
        self.refresh_data_if_needed().unwrap();
        self.recalculate();
        self.display_results();
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
        // refresh
        // quit | exit
        let mut cmd_parts: std::str::SplitWhitespace<'_> = cmd.split_whitespace();

        match cmd_parts.next() {
            Some("help") => {
                println!("Valid commands are `command <type>` but don't type the <> chars...");
                println!("help - display valid commands");
                println!("quit or exit - quit the program");
                println!("volume <float> - set the minimum trading volume");
                println!("profit <float> - set the minimum trading profit");
                println!("top <integer> - display the top <int> trades per currency");
                println!("refresh - refresh the data if necessary")
            }
            Some("volume") => {
                if let Some(arg) = cmd_parts.next() {
                    if let Ok(vol) = arg.parse::<f64>() {
                        self.min_volume = vol;
                        println!("Min volume set {vol}");
                        self.recalculate();
                        self.display_results();
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
                        self.recalculate();
                        self.display_results();
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
                        self.display_results();
                    } else {
                        println!("Invalid value for top, use int.")
                    }
                }
            }
            Some("refresh") => {
                if let Err(e) = self.refresh_data_if_needed() {
                    eprintln!("Error refreshing data: {e}");
                } else {
                    self.display_results();
                }
            }
            Some("quit") | Some("exit") => self.should_quit = true,
            Some(other) => println!("Unknown command: {other}"),
            None => {}
        };
    }

    pub fn refresh_data_if_needed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.as_ref().unwrap();

        let most_recent_snapshot =
            api::get_exchange_snapshot(client).expect("Couldn't get newest snapshot: ");
        let newest_snapshot = most_recent_snapshot.epoch;

        if self.current_snapshot != Some(newest_snapshot) {
            println!("Refreshing data, newest snapshot {newest_snapshot}");

            let cached_snapshots: Vec<fs::DirEntry> = api::list_all_snapshots(&self.data_path)?;

            self.current_records = Some(api::get_freshest_data(
                most_recent_snapshot.epoch,
                &cached_snapshots,
                client,
                &self.data_path,
            ));
            self.current_snapshot = Some(newest_snapshot);
        } else {
            println!("Already have newest snapshot {}", newest_snapshot);
        }

        Ok(())
    }

    pub fn recalculate(&mut self) {
        // TODO: This bit shouldn't print as part of the logic
        let current_records = &self.current_records.as_ref().unwrap();
        self.base_rates = logic::get_base_prices(current_records);

        let valid_bridges: Vec<_> = current_records
            .iter()
            .filter(|exch| exch.volume >= self.min_volume && exch.is_valid_bridge())
            .collect();

        let (hub_to_bridge, bridge_to_hub) = logic::build_hub_bridge_maps(&valid_bridges);
        let mut potential_profits = logic::build_bridges(&hub_to_bridge, &bridge_to_hub);

        potential_profits
            .retain(|elem| logic::eval_profit(elem, &self.base_rates, self.min_profit_frac));
        potential_profits.sort_by(|a, b| b.3.abs().total_cmp(&a.3.abs()));
        self.results = Some(potential_profits.clone());
    }

    pub fn display_results(&self) {
        println!("Divine to Exalt ratio {:?}", &self.base_rates.div_to_exalt);
        println!("Divine to Chaos ratio {:?}", &self.base_rates.div_to_chaos);
        println!("Chaos to Exalt ratio {:?}", &self.base_rates.chaos_to_exalt);

        for currency in [TradingCurrencyType::Divine, TradingCurrencyType::Chaos] {
            println!("Top {} vals:", currency);
            let rate = match currency {
                TradingCurrencyType::Divine => self.base_rates.div_to_exalt,
                TradingCurrencyType::Chaos => self.base_rates.chaos_to_exalt,
                TradingCurrencyType::Exalt => 1.0 / self.base_rates.div_to_exalt,
                TradingCurrencyType::Other => 0.0,
            };
            let top_n_items =
                logic::get_top_items(self.results.as_ref().unwrap(), &currency, self.top);
            for elem in top_n_items {
                println!(
                    "{curr1:<7.7} -> {bridge:^25.25} -> {curr2:<8} | effective ratio: {ratio:>5.1} | profit {profit:>5.1} exalt",
                    curr1 = elem.0,
                    bridge = elem.1,
                    curr2 = elem.2,
                    ratio = elem.3,
                    profit = elem.3 - rate
                )
            }
        }
    }
}
