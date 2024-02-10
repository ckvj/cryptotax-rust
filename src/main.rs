mod funcs;
use std::env;
use std::error::Error;
use colored::Colorize;


/// Given filepath to config file
fn main() -> Result<(), Box<dyn Error>> {
    
    // Config
    let config_filepath: String = collect_config_filepath();
    let config = match funcs::config::build_config(&config_filepath) {
        Ok(config) => config,
        Err(ConfigParseError) => panic!("{}", ConfigParseError.to_string().on_purple()), 
    };

    // Import Trades
    let trades = funcs::import_trades::import_trades(&config).unwrap();

    // Sales
    let sale_events = funcs::process_trades::get_sale_events(trades, &config);
    cli_table::print_stdout(&sale_events).unwrap();
    println!("{}", sale_events.len());

    let annual_summary = funcs::process_trades::get_annual_summary(&sale_events);
    
    annual_summary.
        iter()
        .for_each(|(k,v)| println!("{:?} {:?}", k,v));

    Ok(())
    
}

fn collect_config_filepath() -> String {
    match env::args().nth(1) {
        Some(filepath) => filepath,
        None => panic!("Please provide a filepath to the config file as an argument"),
    }
}