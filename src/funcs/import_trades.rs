
use csv::{ReaderBuilder, StringRecord};

use std::collections::HashMap;
use std::error::Error;

use crate::funcs::config::Config;
use crate::funcs::trade::Trade;


#[derive(serde::Deserialize, Debug)]
pub struct CsvRecord {
    pub timestamp: String,
    pub txn_type: String,
    pub base_asset: String,
    pub base_asset_amount: String,
    pub quote_asset: String,
    pub quote_asset_amount: String,
}


pub fn import_trades(config: &Config) -> Result<HashMap<String, Vec<Trade>>, Box<dyn Error>> {

    // File import
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(&config.filepath)?;
    let updated_headers = replace_header_names(rdr.headers()?, &config.csv_columns);
    rdr.set_headers(updated_headers.to_owned());


    let mut sorted_trades: HashMap<String, Vec<Trade>> = HashMap::new(); // Where trades are stored
    
    for result in rdr.deserialize() {
        
        let trade: Trade = Trade::new(result?, config)?;
        let asset_name = trade.base_asset.to_owned();

        let trades = sorted_trades.entry(asset_name).or_default();
        trades.push(trade);
    }

    Ok(sorted_trades)
}



/// Updates the headers in a `StringRecord` with the corresponding values from a HashMap, ignore others
fn replace_header_names(headers: &StringRecord, column_map: &HashMap<String, String>) -> StringRecord {
    let mut updated_headers: Vec<String> = Vec::new();

    for header in headers.iter() {
        if let Some(new_header) = column_map.get(header) {
            updated_headers.push(String::from(new_header))
        } else {
            updated_headers.push(String::from(header));
        }
    }
    StringRecord::from(updated_headers)
}
