/// Imports Section <> Value names from a INI file
use ini::Ini;
use std::collections::HashMap;
use std::error::Error;

/// Enum of three different types of analysis types
#[derive(Debug, Default, Clone)]
pub enum AccountingType {
    #[default]
    LIFO,
    FIFO,
    HIFO,
}

impl AccountingType {
    /// Matches an accounting type string to an `AccountingType` enum
    fn match_accounting_type(accounting_type: &str) -> Self {
        
        // Question: I try to create a copy and match to .lowercase_() so I don't need to write out three versions, but cannot figure out clean solution.
        match accounting_type {
            "LIFO" | "Lifo" | "lifo" => Self::LIFO,
            "FIFO" | "Fifo" | "fifo" => Self::FIFO,
            "HIFO" | "Hifo" | "hifo" => Self::HIFO,
            _ => panic!("CANNOT MATCH ACCOUNTING TYPE"), // Question: Panic best here since no easy recovery?
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub accounting_type: AccountingType,
    pub filepath: String,
    pub csv_columns: HashMap<String, String>,
    pub buy_txn_types: Vec<String>,
    pub sell_txn_types: Vec<String>,
}

pub fn build_config(config_filepath: &str) -> Result<Config, Box<dyn Error>> {
    let mut config = Config::default();

    let i = match Ini::load_from_file(config_filepath) {
        Ok(i) => i,
        Err(_) => return Err("ERROR: Can't find file".into())
    };

    for (ind, section) in i.sections().enumerate() {
        match section {
            Some("accounting_type") => {
                let accounting_str = &i["accounting_type"]["accounting_type"];
                config.accounting_type = AccountingType::match_accounting_type(accounting_str)
            }
            Some("file_info") => {
                config.filepath =
                    format!("{}{}", &i["file_info"]["dir"], &i["file_info"]["filename"]);
            }
            Some("csv_columns") => {
                config.csv_columns = get_map_swap(&i["csv_columns"]);
            }
            Some("buy_txn_types") => {
                config.buy_txn_types = get_vector_values(&i["buy_txn_types"]);
            }
            Some("sell_txn_types") => {
                config.sell_txn_types = get_vector_values(&i["sell_txn_types"]);
            }
            None => {
                if ind == 0 {
                    // Ignore General Section, which is always returned first
                    dbg!("No outlier fields");
                    continue;
                } else {
                    panic!("Uncharacterized Error, attempted import on None type on config file");
                }
            }
            _ => {
                println!("Attempt to import unknown section");
                dbg!(&section);
            }
        }
    }
    Ok(config)
}

/// Return vector of INI Section values
fn get_vector_values(section: &ini::Properties) -> Vec<String> {
    section
        .iter()
        .map(|(_, value)| String::from(value))
        .collect()
}

/// Return Hashmap<value,key> for a ini file section
fn get_map_swap(section: &ini::Properties) -> HashMap<String, String> {
    section
        .iter()
        .map(|(key, value)| (String::from(value), String::from(key)))
        .collect()
}
