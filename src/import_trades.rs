use chrono::NaiveDateTime;
use csv::{ReaderBuilder, StringRecord};
use std::{collections::HashMap};
use rust_decimal::Decimal;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum TxnType {
    Buy,
    Sale,
    Other,
}

#[derive(Debug, Clone)]
pub struct Trade {
    trade_time: NaiveDateTime,
    txn_type: TxnType,
    base_asset: String,
    base_asset_amount: Decimal,
    quote_asset: String,
    quote_asset_amount: Decimal,
    remaining: Decimal,
    unix_time: i64
}

impl Trade {
    pub fn new (
        time_string: String, 
        txn_type_str: String,
        base_asset: String,
        base_asset_amount: Decimal,
        quote_asset: String,
        quote_asset_amount: Decimal,
        config: &Config,
    ) -> Self {
        
        // DateTime
        let remaining: Decimal = base_asset_amount;
        let trade_time = NaiveDateTime::parse_from_str(&time_string,"%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        let unix_time: i64 = NaiveDateTime::timestamp(&trade_time).try_into().unwrap();

        // Txn Type
        let txn_type: TxnType;
        if contains_in_vector(&txn_type_str, &config.buy_txn_types) {
            txn_type = TxnType::Buy;
       } else if contains_in_vector(&txn_type_str, &config.sell_txn_types) {
            txn_type = TxnType::Sale;
       } else {
            txn_type = TxnType::Other;
       }
       

       // Return
        Self {
            trade_time,
            txn_type,
            base_asset,
            base_asset_amount,
            quote_asset,
            quote_asset_amount,
            remaining,
            unix_time,
        }
    }
}


#[derive(Debug)]
pub struct Asset {
    name: String,
    trades: Vec<Trade>,
}

pub fn import_trades(config: &Config) -> HashMap<String, Asset> {

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(&config.filepath)
        .expect("error opening file");

    // Update Headers
    let headers = rdr.headers().unwrap();
    let updated_headers = update_headers(headers, &config.csv_columns);
    let header_map = build_header_map(&updated_headers);
    rdr.set_headers(updated_headers.clone());

    let mut sorted_trades: HashMap<String, Asset> = HashMap::new();

    for record in rdr.records() {
        let record = record.unwrap();

        let timestamp = String::from(&record[*header_map.get("timestamp").unwrap()]);
        let txn_type = String::from(&record[*header_map.get("txn_type").unwrap()]);
        let base_asset = String::from(&record[*header_map.get("base_asset").unwrap()]);
        let base_asset_amount = Decimal::try_from(&record[*header_map.get("base_asset_amount").unwrap()]).unwrap();
        let quote_asset = String::from(&record[*header_map.get("quote_asset").unwrap()]);
        let quote_asset_amount = Decimal::try_from(&record[*header_map.get("quote_asset_amount").unwrap()]).unwrap();

        let trade = Trade::new(timestamp, txn_type, base_asset.to_owned() ,base_asset_amount,quote_asset, quote_asset_amount, config);

        if sorted_trades.contains_key(&base_asset) {
            sorted_trades.entry(base_asset).and_modify(|asset| asset.trades.push(trade.to_owned()));
        } else {
            sorted_trades.insert(base_asset.to_owned(), Asset {name: base_asset.to_owned(), trades: vec![trade]});
        }
    }

    sorted_trades

}


fn update_headers(headers: &StringRecord, column_map: &HashMap<String,String>) -> StringRecord {

    let mut updated_headers: Vec<String> = Vec::new();
    
    for header in headers.iter() {
        if let Some(new_header) =  column_map.get(header) {
            updated_headers.push(String::from(new_header))
        } else {
            updated_headers.push(String::from(header));
        }
    }
    StringRecord::from(updated_headers)
}

fn build_header_map(headers: &StringRecord) -> HashMap<&str, usize> {

    let mut header_map = HashMap::new();
    
    for (i,v) in headers.iter().enumerate() { 
        match v {
            "timestamp" | "txn_type" | "base_asset" | "base_asset_amount" | "quote_asset" | "quote_asset_amount" => {
                header_map.insert(v,i);
            },
            _ => (),
        }
    }
    header_map.clone()
}

fn contains_in_vector(string: &str, string_vector: &Vec<String>) -> bool {
    for s in string_vector {
        if string.contains(s) {
            return true;
        }
    }
    return false;
}
