/// Imports Section <> Value names from a INI file

use ini::Ini;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{error::Error, fmt::Debug};


#[derive(thiserror::Error, Debug)]
pub enum ConfigParseError {
    #[error("ERROR: Could not load ini file from provided input filepath.\nMSG: {0}")]
    IniLoadError(#[from] ini::Error),
    #[error("Could not derive intended transaction filepath from config input")]
    FilepathError,
}


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
    fn match_accounting_type(accounting_type: &str) -> Option<Self> {
        match accounting_type.to_uppercase().as_str() {
            "LIFO" => Some(Self::LIFO),
            "FIFO" => Some(Self::FIFO),
            "HIFO" => Some(Self::HIFO),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub accounting_type: AccountingType,
    pub filepath: PathBuf,
    pub csv_columns: HashMap<String, String>,
    pub buy_txn_types: Vec<String>,
    pub sell_txn_types: Vec<String>,
    // pub venues: Option<Vec<String>>, // Optional Field
}

pub fn build_config(config_filepath: PathBuf) -> Result<Config, ConfigParseError> {
    let mut config = Config::default();

    let ini_file = Ini::load_from_file(config_filepath).map_err(ConfigParseError::IniLoadError)?;

    for (index, section) in ini_file.sections().enumerate() {
        match section {
            Some("accounting_type") => {
                let accounting_str = &ini_file[section]["accounting_type"];
                config.accounting_type = match AccountingType::match_accounting_type(accounting_str)
                {
                    Some(a) => a,
                    None => panic!("Cannot match accounting type")
                };
            }
            Some("file_info") => {
                let dir = &ini_file[section]["dir"];
                let filename = &ini_file[section]["filename"];

                if dir.is_empty() || filename.is_empty() {
                    return Err(ConfigParseError::FilepathError);
                }

                config.filepath = PathBuf::from(dir).join(filename);
            }
            Some("csv_columns") => {
                config.csv_columns = get_map_and_swap(&ini_file[section]);
            }
            Some("buy_txn_types") => {
                config.buy_txn_types = string_to_vec(&ini_file[section]["buys"]);
            }
            Some("sell_txn_types") => {
                config.sell_txn_types = string_to_vec(&ini_file[section]["sells"]);
            }
            // Some("venues") => {
            //     let col_name: String = String::from(&ini_file[section]["column_name"]);
            //     config.csv_columns.insert(col_name, "venue".to_string());
            //     config.venues = Some(string_to_vec(&ini_file[section]["venues"]));
            // }
            None => {
                // Ignore General Section, which is always returned first (i=0)
                if index != 0 {
                    panic!("Uncharacterized Error, attempted import on None type on config file");
                }
            }
            _ => {
                println!("Attempt to import unknown section: {}", section.unwrap());
            }
        }
    }
    Ok(config)
}

// /// Return vector of INI Section values
// fn get_vector_values(section: &ini::Properties) -> Vec<String> {
//     section
//         .iter()
//         .map(|(_, value)| String::from(value))
//         .collect()
// }

fn string_to_vec(string_: &str) -> Vec<String> {

    string_
    .split(',')
    .map(|s| s.trim().to_string())
    .collect()
}


/// Return Hashmap<value,key> for a ini file section
fn get_map_and_swap(section: &ini::Properties) -> HashMap<String, String> {
    section
        .iter()
        .map(|(key, value)| (String::from(value), String::from(key)))
        .collect()
}
