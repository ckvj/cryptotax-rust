mod funcs;
use std::env;
use std::error::Error;

/// Given filepath to config file
fn main() -> Result<(), Box<dyn Error>> {
    let config_filepath: String = collect_config_filepath().unwrap();
    let config = match funcs::config::build_config(&config_filepath) {
        Ok(config) => config,
        Err(e) => panic!("Error: {}", e), // Question: I cannot seem to unpack the different types of Box Errors that can be returned, so need to panic
    };

    let trades = funcs::import_trades::import_trades(&config).unwrap();

    let sale_events = funcs::process_trades::get_sale_events(trades, &config);

    cli_table::print_stdout(&sale_events).unwrap();
    println!("{}", sale_events.len());

    let annual_summary = funcs::process_trades::get_annual_summary(&sale_events);
    
    annual_summary.
        iter()
        .for_each(|(k,v)| println!("{:?} {:?}", k,v));

    Ok(())
    
    
}

fn collect_config_filepath() -> Option<String> {
    match env::args().nth(1){
        Some(filepath) => Some(filepath),
        None => panic!("Please provide a filepath to the first argument"), // Question: How could I pass something back rather than panic?
    }
}