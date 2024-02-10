
use csv::{ReaderBuilder, StringRecord};

use std::collections::HashMap;
use std::error::Error;

use crate::funcs::config::Config;
use crate::funcs::trade::{Asset, Trade, process_record_into_trade};

pub fn import_trades(config: &Config) -> Result<HashMap<String, Asset>, Box<dyn Error>> {
    
    // Read Config
    let mut rdr = ReaderBuilder::new()
                                .has_headers(true)
                                .from_path(&config.filepath)?;

    // Update Headers
    let headers: &StringRecord = rdr.headers()?;
    let updated_headers = replace_header_names(headers, &config.csv_columns);
    rdr.set_headers(updated_headers.to_owned());
    let header_indices = build_header_indicies_map(&updated_headers);

    // Process Rows
    let mut sorted_trades: HashMap<String, Asset> = HashMap::new();
    for record in rdr.records() {
        let trade: Trade = process_record_into_trade(&record?, &header_indices, config)?;

        let asset_name = trade.base_asset.to_owned();

        if sorted_trades.contains_key(&asset_name) {
            sorted_trades
                .entry(asset_name)
                .and_modify(|asset| asset.trades.push(trade.to_owned()));
        } else {
            sorted_trades.insert(
                asset_name.to_owned(),
                Asset {
                    name: asset_name.to_owned(),
                    trades: vec![trade],
                },
            );
        }
    }
    Ok(sorted_trades)
}



/// Updates the headers in a `StringRecord` with the corresponding values from a HashMap, ignore others
fn replace_header_names(
    headers: &StringRecord,
    column_map: &HashMap<String, String>,
) -> StringRecord {
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

/// Return index of selected header names
fn build_header_indicies_map(headers: &StringRecord) -> HashMap<&str, usize> {
    let mut header_map = HashMap::new();

    for (i, v) in headers.iter().enumerate() {
        match v {
            "timestamp" | "txn_type" | "base_asset" | "base_asset_amount" | "quote_asset"
            | "quote_asset_amount" => {
                header_map.insert(v, i);
            }
            // "venue" => {header_map.insert(v, i);}
             _ => (),
        }
    }
    header_map
}