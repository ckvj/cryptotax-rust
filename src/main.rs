use std::{env, error::Error};

mod config;


fn main() {

<<<<<<< Updated upstream
    let args: Vec<String> = env::args().collect();
    let config_filepath: String = String::from(&args[1]);

    let config = config::build_config(&config_filepath);
    println!("{:?}", config);
=======
    // let args: Vec<String> = env::args().collect();
    // let config_filepath: String = String::from(&args[1]);

    let config_filepath = collect_config_filepath().unwrap();

    let config = config::build_config(&config_filepath);

    let trades = import_trades::import_trades(&config);
    println!("{:?}", &trades["BTC"])
>>>>>>> Stashed changes
    
}

fn collect_config_filepath() -> Option<String> {
    let filepath = env::args().nth(1);

    match filepath {
        Some(filepath) => Some(filepath),
        None => panic!("Please provide a filepath to the first argument"),
    }
    
}
