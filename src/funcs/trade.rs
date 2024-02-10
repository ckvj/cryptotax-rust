use chrono::{NaiveDateTime, ParseError};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::error::Error;
use std::collections::HashMap;

use crate::funcs::config::Config;
use crate::funcs::txn_type::TxnType;

#[derive(thiserror::Error, Debug)]
pub enum ValueParseError {
    #[error("ERROR: Could not parse provided datetime: {0}")]
    DatetimeFormatParseError(String),
    #[error("ERROR: Could not parse provided amount: {value}")]
    DecimalParseError {
        value: String,
        #[source]
        source: rust_decimal::Error,
    }
}

/// Trade is created from a StringRecord in the function process_record_into_trade
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
    // pub venue: Option<String>
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
        // venue: Option<String>,
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

        // TxnType interpolation
        let base_asset_amount: Decimal = base_asset_amount.parse::<Decimal>().
            map_err(|e| ValueParseError::DecimalParseError { value: base_asset_amount.clone(), source: e })?;

        let quote_asset_amount: Decimal = quote_asset_amount.parse::<Decimal>().
            map_err(|e| ValueParseError::DecimalParseError { value: quote_asset_amount.clone(), source: e })?;
        
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
            // venue,
        })
    }
}


pub fn process_record_into_trade(record: HashMap<String, String>, config:&Config) -> Result<Trade, Box<dyn Error>> {

    let time_string = record.get("timestamp").unwrap().to_string();
    let txn_type_string = record.get("txn_type").unwrap().to_string();
    let base_asset = record.get("base_asset").unwrap().to_string();
    let quote_asset = record.get("quote_asset").unwrap().to_string();
    let base_asset_amount = record.get("base_asset_amount").unwrap().to_string();
    let quote_asset_amount = record.get("quote_asset_amount").unwrap().to_string();
    
    // let venue = match config.venues.is_some() {
    //     false => None,
    //     true => Some(get_value_from_record("venue", record, header_indices)?),
    // };
    
    
    Trade::new(
        time_string,
        txn_type_string,
        base_asset,
        base_asset_amount,
        quote_asset,
        quote_asset_amount,
        config,
        // venue,
    )
}


pub fn parse_datetime_string(datetime: &str) -> Result<NaiveDateTime, ValueParseError> {
    // Common Date Formats
    let common_formats = [
        "%Y-%m-%dT%H:%M:%S%.3fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
    ];
    
    for fmt in common_formats.iter() {
        if let Ok(dt) = NaiveDateTime::parse_from_str(datetime, fmt) {
            return Ok(dt);
        }
    }
    Err(ValueParseError::DatetimeFormatParseError(datetime.to_string()))
}

// pub fn get_value_from_record(field: &str, record: &StringRecord, header_indices: &HashMap<&str, usize>) -> Result<String, Box<dyn Error>> {
//     // dbg!(&field);
//     // dbg!(&record);
//     // dbg!(&header_indices);
//     let header_index = header_indices.get(field).unwrap_or_else(|| panic!("Error parsing field '{}' in record {:?}", &field, &record));
//     Ok(record.get(*header_index).unwrap().to_string())
// }