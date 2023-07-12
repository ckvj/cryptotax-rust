use chrono::NaiveDateTime;
use csv::{ReaderBuilder, StringRecord};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::collections::HashMap;
use std::error::Error;

use crate::funcs::config::Config;

/// TxnType is exclusive and classified all trades as Buy, Sale, or Other, where Other can includes non-capital gain events
#[derive(Debug, Clone, PartialEq)]
pub enum TxnType {
    Buy,
    Sale,
    Other,
}

impl TxnType {
    /// Returns txn_type enum for provided txn_type string and available classification vectors
    pub fn return_txn_type(
        match_string: &String,
        buy_vector: &Vec<String>,
        sell_vector: &Vec<String>,
    ) -> TxnType {
        match match_string {
            _ if contains_in_vector(&match_string, buy_vector) => TxnType::Buy, //Question: Why does analyzer recommend I remove the & in the &match_string?
            _ if contains_in_vector(&match_string, sell_vector) => TxnType::Sale,
            _ => TxnType::Other,
        }
    }
}

/// Trade is what's imported from csv files. Some fields read from CSV and others are derived.
#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_time: NaiveDateTime,
    pub txn_type: TxnType,
    pub base_asset: String,
    pub base_asset_amount: Decimal,
    pub quote_asset: String,
    pub quote_asset_amount: Decimal,
    pub remaining: Decimal,
    pub unix_time: i64,
    pub price: f32,
}

impl Trade {
    pub fn new(
        time_string: String,
        txn_type_string: String,
        base_asset: String,
        base_asset_amount: Decimal,
        quote_asset: String,
        quote_asset_amount: Decimal,
        config: &Config,
    ) -> Result<Self, Box<dyn Error>> {
        
        // DateTime derivations
        let trade_time = match NaiveDateTime::parse_from_str(&time_string, "%Y-%m-%dT%H:%M:%S%.3fZ"){
            Ok(trade_time) => trade_time,
            Err(_) => return Err("Cannot parse date format".into()), // Question: How can I include the specific time_string in error reponse?
        };
        
        let unix_time: i64 = NaiveDateTime::timestamp(&trade_time);

        // TxnType
        let txn_type = TxnType::return_txn_type(
            &txn_type_string,
            &config.buy_txn_types,
            &config.sell_txn_types,
        );

        // Price
        let price = quote_asset_amount
            .checked_div(base_asset_amount)
            .unwrap()
            .to_f32()
            .unwrap();

        let remaining: Decimal = base_asset_amount;

        // Return
        Ok(Self {
            trade_time,
            txn_type,
            base_asset,
            base_asset_amount,
            quote_asset,
            quote_asset_amount,
            remaining,
            unix_time,
            price,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub trades: Vec<Trade>,
}

pub fn import_trades(config: &Config) -> Result<HashMap<String, Asset>, Box<dyn Error>> {
    let mut rdr = match ReaderBuilder::new().has_headers(true).from_path(&config.filepath) {
            Ok(rdr) => rdr,
            Err(_) => return Err("ERROR opening file from provided filepath".into()), // Question: Better way to do this? Would be nice to match different types of errors in Main, but can't seem to match on error messages in a Box
        };

    // Update Headers
    let headers: &StringRecord = rdr.headers()?;
    let updated_headers = replace_header_names(headers, &config.csv_columns);
    rdr.set_headers(updated_headers.to_owned());
    let header_indices = build_header_indicies_map(&updated_headers);

    let mut sorted_trades: HashMap<String, Asset> = HashMap::new();

    // Process Rows
    for record in rdr.records() {
        let record = record.unwrap();

        let time_string = String::from(&record[*header_indices.get("timestamp").unwrap()]); // Question: tried ".to_string()" here and it didn't work, said it was type &String. Why?
        let txn_type_string = String::from(&record[*header_indices.get("txn_type").unwrap()]);
        let base_asset = String::from(&record[*header_indices.get("base_asset").unwrap()]);
        let base_asset_amount = Decimal::try_from(&record[*header_indices.get("base_asset_amount").unwrap()]).unwrap();
        let quote_asset = String::from(&record[*header_indices.get("quote_asset").unwrap()]);
        let quote_asset_amount = Decimal::try_from(&record[*header_indices.get("quote_asset_amount").unwrap()]).unwrap();

        let asset_name = base_asset.to_owned();

        let trade = Trade::new(
            time_string,
            txn_type_string,
            base_asset,
            base_asset_amount,
            quote_asset,
            quote_asset_amount,
            config,
        )?; // Question: Think the ? is fine here since it should propagate more specific error message from before.

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
            _ => (),
        }
    }
    header_map
}

/// Returns bool for string contained in any string in string_vector
pub fn contains_in_vector(string: &str, string_vector: &Vec<String>) -> bool {
    for s in string_vector {
        if string.contains(s) {
            return true;
        }
    }
    false
}
