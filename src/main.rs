mod funcs;
use std::env;
use cli_table::{print_stdout, WithTitle};

/// Given filepath to config file
fn main() {
    let config_filepath: String = collect_config_filepath().unwrap();
    let config = funcs::config::build_config(&config_filepath);

    let trades = funcs::import_trades::import_trades(&config);

    let sale_events = funcs::process_trades::get_sale_events(trades, &config);
    
    let annual_summary = funcs::process_trades::get_annual_summary(&sale_events);
    println!("{:?}", annual_summary);

}

fn collect_config_filepath() -> Option<String> {
    let filepath = env::args().nth(1);

    match filepath {
        Some(filepath) => Some(filepath),
        None => panic!("Please provide a filepath to the first argument"),
    }
    
}
