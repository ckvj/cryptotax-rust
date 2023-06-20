// TODO!: Figure how to ensure all fields are captured & ingested properly

use ini::Ini;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Config {
    accouting_type: String,
    pub filepath: String,
    csv_columns: HashMap<String,String>,
    buy_txn_types: Vec<String>,
    sell_txn_types: Vec<String>,
}

pub fn build_config (config_filepath: &str) -> Config {
    
    let mut config = Config::default();

    let i = Ini::load_from_file(config_filepath).expect("Can't find file");
    
    for section in i.sections() {
        match section {
            Some("accounting_type") => {
                config.accouting_type = i["accounting_type"]["accounting_type"].to_string();
            },
            Some("file_info") => {
                config.filepath = format!("{}{}", &i["file_info"]["dir"], &i["file_info"]["filename"]);
            },
            Some("csv_columns") => {
                config.csv_columns = get_map(&i, "csv_columns");
            },
            Some("buy_txn_types") => {
                config.buy_txn_types = get_vector_values(&i, "buy_txn_types");
            },
            Some("sell_txn_types") => {
                config.sell_txn_types = get_vector_values(&i, "sell_txn_types");
            },
            None => println!("Returned None Type"),
            _ => println!("No match for {:?}", section),
        }
    }
    config
}



fn get_vector_values(ini: &Ini, section: &str) -> Vec<String> {
    let values: Vec<String> = ini
        .section(Some(section))
        .unwrap()
        .iter()
        .map(|(_, value)| String::from(value))
        .collect();

    values
}

fn get_map(ini: &Ini, section: &str) -> HashMap<String, String> {
    ini
        .section(Some(section))
        .unwrap()
        .iter()
        .map(|(key, value)| (String::from(key), String::from(value)))
        .collect()
}

