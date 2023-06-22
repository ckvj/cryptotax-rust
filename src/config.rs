use ini::Ini;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
<<<<<<< Updated upstream
    accouting_type: String,
    filepath: String,
    csv_columns: HashMap<String,String>,
    buy_txn_types: Vec<String>,
    sell_txn_types: Vec<String>,
=======
    pub accouting_type: String,
    pub filepath: String,
    pub csv_columns: HashMap<String,String>,
    pub buy_txn_types: Vec<String>,
    pub sell_txn_types: Vec<String>,
>>>>>>> Stashed changes
}

pub fn build_config (config_filepath: &str) -> Config {
    
    // Initialize empty struct which will be overwritten
    let mut config = Config {
        accouting_type: "".to_string(),
        filepath: "".to_string(),
        csv_columns: HashMap::new(),
        buy_txn_types: Vec::new(),
        sell_txn_types: Vec::new(),
    };

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
                config.csv_columns = get_map_swap(&i, "csv_columns");
            },
            Some("buy_txn_types") => {
                config.buy_txn_types = get_vector_values(&i, "buy_txn_types");
            },
            Some("sell_txn_types") => {
                config.sell_txn_types = get_vector_values(&i, "sell_txn_types");
            },
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