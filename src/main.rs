mod funcs;
use std::env;
use std::error::Error;
use colored::Colorize;
use std::path::{Path, PathBuf};

#[allow(warnings, dead_code)]

/// Given filepath to config file
fn main() -> Result<(), Box<dyn Error>> {
    
    // Config
    let config_filepath: PathBuf = collect_config_filepath()?;
    let config = match funcs::config::build_config(config_filepath) {
        Ok(config) => config,
        Err(ConfigParseError) => panic!("{}", ConfigParseError.to_string().on_purple()), 
    };

    // Import Trades
    let trades: std::collections::HashMap<String, funcs::asset::Asset> = funcs::import_trades::import_trades(&config).unwrap();

    let sale_events = funcs::process_trades::get_sale_events(trades, &config);
    
    let df = funcs::process_trades::convert_vec_to_df(&sale_events);

    let grouped_df = df
        .group_by(["Asset Name", "Sell Year"])?
        .select(["Gain-Loss"]).sum().unwrap()
        .sort(&["Asset Name", "Sell Year"], false, false)?;

    let annual_summary = funcs::process_trades::get_annual_summary(&sale_events);
    println!("{}", annual_summary);

    Ok(())
    
}


fn collect_config_filepath() -> Result<PathBuf, String> {
    match env::args().nth(1) {
        Some(filepath) => Ok(PathBuf::from(filepath)), // TODO: Check if file exists
        None => Err("Please provide a filepath to the config file as an argument".to_string()),
    }
}
