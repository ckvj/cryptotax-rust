use std::env;
mod config;


fn main() {

    let args: Vec<String> = env::args().collect();
    let config_filepath: String = String::from(&args[1]);

    let config = config::build_config(&config_filepath);
    println!("{:?}", config);
    
}

