use std::{
    fs,
    io::{self, Error, Write},
    path::Path,
};

use reqwest::blocking::Client;

use crate::{
    api, logic,
    models::logic_models::{TradingCurrencyRates, TradingCurrencyType},
};

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
        println!("Use help for valid commands");
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
            Some("refresh") => {
                self.refresh_data();
            }
            Some("quit") | Some("exit") => self.should_quit = true,
            Some(other) => println!("Unknown command: {other}"),
            None => {}
        };
    }
    pub fn refresh_data(&mut self) {
        let client = self.client.as_ref().unwrap();

        let most_recent_snapshot =
            api::get_exchange_snapshot(client).expect("Couldn't get newest snapshot: ");

        println!(
            "Most recent snapshot number: {}",
            &most_recent_snapshot.epoch
        );

        let data_path: &Path = Path::new("data");

        let cached_snapshots: Vec<fs::DirEntry> = api::list_all_snapshots(data_path);

        let newest_pairs = api::get_freshest_data(
            most_recent_snapshot.epoch,
            &cached_snapshots,
            client,
            data_path,
        );

        let base_rates = logic::get_base_prices(&newest_pairs);
        println!("Divine to Exalt ratio {:?}", &base_rates.div_to_exalt);
        println!("Divine to Chaos ratio {:?}", &base_rates.div_to_chaos);
        println!("Chaos to Exalt ratio {:?}", &base_rates.chaos_to_exalt);

        // TODO: below this line should probably be in a new function as
        // this is all now program logic that isn't related to refreshing data.

        let valid_bridges: Vec<_> = newest_pairs
            .into_iter()
            .filter(|exch| exch.volume >= self.min_volume && exch.is_valid_bridge())
            .collect();

        let (hub_to_bridge, bridge_to_hub) = logic::build_hub_bridge_maps(&valid_bridges);
        let mut potential_profits = logic::build_bridges(&hub_to_bridge, &bridge_to_hub);

        potential_profits
            .retain(|elem| logic::eval_profit(elem, &base_rates, self.min_profit_frac));
        potential_profits.sort_by(|a, b| b.3.abs().total_cmp(&a.3.abs()));

        self.display_results(potential_profits, &base_rates);
    }
    pub fn display_results(
        &self,
        results: Vec<(TradingCurrencyType, String, TradingCurrencyType, f64)>,
        base_rates: &TradingCurrencyRates,
    ) {
        for currency in [TradingCurrencyType::Divine, TradingCurrencyType::Chaos] {
            println!("Top {} vals:", currency);
            let rate = match currency {
                TradingCurrencyType::Divine => base_rates.div_to_exalt,
                TradingCurrencyType::Chaos => base_rates.chaos_to_exalt,
                TradingCurrencyType::Exalt => 1.0 / base_rates.div_to_exalt,
                TradingCurrencyType::Other => 0.0,
            };
            let to_print = logic::get_top_items(&results, &currency, self.top);
            for elem in to_print {
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
