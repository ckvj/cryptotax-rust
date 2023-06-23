// TODO!: Figure how to ensure all fields are captured & ingested properly

use ini::Ini;
use std::collections::HashMap;


#[derive(Debug, Default)]
pub enum AccountingType {
    #[default]
    LIFO,
    FIFO,
    HIFO,
}


#[derive(Debug, Default)]
pub struct Config {
    pub accouting_type: AccountingType,
    pub filepath: String,
    pub csv_columns: HashMap<String,String>,
    pub buy_txn_types: Vec<String>,
    pub sell_txn_types: Vec<String>,
}

pub fn build_config (config_filepath: &str) -> Config {
    
    let mut config = Config::default();

    let i = Ini::load_from_file(config_filepath).expect("Can't find file");
    
    for section in i.sections() {
        match section {
            Some("accounting_type") => {
                let accounting_str = &i["accounting_type"]["accounting_type"];
                config.accouting_type = match_accounting_type(accounting_str)
            },
            Some("file_info") => {
                config.filepath = format!("{}{}", &i["file_info"]["dir"], &i["file_info"]["filename"]);
            },
            Some("csv_columns") => {
                config.csv_columns = get_map_swap(&i, "csv_columns");
            },
            Some("buy_txn_types") => {
                config.buy_txn_types = get_vector_values(&i, "buy_txn_types");
            },
            Some("sell_txn_types") => {
                config.sell_txn_types = get_vector_values(&i, "sell_txn_types");
            },
            None => (),
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

fn get_map_swap(ini: &Ini, section: &str) -> HashMap<String, String> {
    ini
        .section(Some(section))
        .unwrap()
        .iter()
        .map(|(key, value)| (String::from(value), String::from(key)))
        .collect()
}

fn match_accounting_type(accounting_type: &str) -> AccountingType {

    match accounting_type {
        "LIFO" | "Lifo" | "lifo" => AccountingType::LIFO,
        "FIFO" | "Fifo" | "fifo" => AccountingType::FIFO,
        "HIFO" | "Hifo" | "hifo" => AccountingType::HIFO,
        _ => panic!("CANNOT MATCH ACCOUNTING TYPE")
    }

}