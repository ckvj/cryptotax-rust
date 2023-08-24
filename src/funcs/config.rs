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
    pub filepath: String,
    pub csv_columns: HashMap<String, String>,
    pub buy_txn_types: Vec<String>,
    pub sell_txn_types: Vec<String>,
}

pub fn build_config(config_filepath: &str) -> Result<Config, Box<dyn Error>> {
    let mut config = Config::default();

    let ini_file = Ini::load_from_file(config_filepath)?;

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
                config.filepath = format!("{}{}", &ini_file[section]["dir"], &ini_file[section]["filename"]);
            }
            Some("csv_columns") => {
                config.csv_columns = get_map_and_swap(&ini_file[section]);
            }
            Some("buy_txn_types") => {
                config.buy_txn_types = get_vector_values(&ini_file[section]);
            }
            Some("sell_txn_types") => {
                config.sell_txn_types = get_vector_values(&ini_file[section]);
            }
            None => {
                // Ignore General Section, which is always returned first (i=0)
                if index != 0 {
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
fn get_map_and_swap(section: &ini::Properties) -> HashMap<String, String> {
    section
        .iter()
        .map(|(key, value)| (String::from(value), String::from(key)))
        .collect()
}
