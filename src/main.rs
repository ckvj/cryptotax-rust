
use std::{env, error::Error};

mod config;
mod import_trades;
mod process_trades;

fn main() {

    
    let config_filepath: String = collect_config_filepath().unwrap();

    let config = config::build_config(&config_filepath);

    let trades = import_trades::import_trades(&config);

    process_trades::get_sale_events(trades.clone(), &config);


}

fn collect_config_filepath() -> Option<String> {
    let filepath = env::args().nth(1);

    match filepath {
        Some(filepath) => Some(filepath),
        None => panic!("Please provide a filepath to the first argument"),
    }
    
}
