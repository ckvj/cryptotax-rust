use chrono::NaiveDateTime;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::error::Error;
use csv::StringRecord;
use std::collections::HashMap;

use crate::funcs::config::Config;
use crate::funcs::txn_type::TxnType;


/// Asset holds Trade events for a given asset
#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub trades: Vec<Trade>,
}

/// Trade is created from a StringRecord in fn process_record_into_trade
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
        base_asset_amount: String,
        quote_asset: String,
        quote_asset_amount: String,
        config: &Config,
    ) -> Result<Self, Box<dyn Error>> {
        
        // DateTime interpolations
        let trade_time = parse_datetime_string(&time_string)?;
        let unix_time: i64 = NaiveDateTime::timestamp(&trade_time);

        // TxnType interpolation
        let txn_type = TxnType::return_txn_type(
            &txn_type_string,
            &config.buy_txn_types,
            &config.sell_txn_types,
        );

        // Base and Quote Asset Amounts
        let base_asset_amount: Decimal = base_asset_amount.parse::<Decimal>()?;
        let quote_asset_amount: Decimal = quote_asset_amount.parse::<Decimal>()?;

        // Price
        let price = quote_asset_amount
            .checked_div(base_asset_amount)
            .and_then(|div_result| div_result.to_f32())
            .unwrap();

        let remaining: Decimal = base_asset_amount; // Field used to represent when a trade was processed in full or part

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

pub fn parse_datetime_string(datetime: &str) -> Result<NaiveDateTime, Box<dyn Error>> {
    // Common Date Formats
    let common_formats = [
        "%Y-%m-%dT%H:%M:%S%.3fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
    ];
    
    for fmt in common_formats.iter() {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&datetime, fmt) {
            return Ok(dt);
        }
    }
    Err(format!("Error parsing datetime {}", datetime).into())
}


pub fn process_record_into_trade(record: &StringRecord, header_indices: &HashMap<&str, usize>, config: &Config) -> Result<Trade, Box<dyn Error>> {

    let time_string = get_value_from_record("timestamp", record, header_indices)?;
    let txn_type_string = get_value_from_record("txn_type", record, header_indices)?;
    let base_asset = get_value_from_record("base_asset", record, header_indices)?;
    let quote_asset = get_value_from_record("quote_asset", record, header_indices)?;
    let base_asset_amount = get_value_from_record("base_asset_amount", record, header_indices)?;
    let quote_asset_amount = get_value_from_record("quote_asset_amount", record, header_indices)?;
    

    Trade::new(
        time_string,
        txn_type_string,
        base_asset,
        base_asset_amount,
        quote_asset,
        quote_asset_amount,
        config,
    )
}

pub fn get_value_from_record(field: &str, record: &StringRecord, header_indices: &HashMap<&str, usize>) -> Result<String, Box<dyn Error>> {
    let header_index = header_indices.get(field).unwrap_or_else(|| panic!("Error parsing field {} in record {:?}", &field, &record));
    Ok(record.get(*header_index).unwrap().to_string())
}