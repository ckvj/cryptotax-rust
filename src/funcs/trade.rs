
use chrono::NaiveDateTime;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::error::Error;

use crate::funcs::config::Config;
use crate::funcs::txn_type::TxnType;
use crate::funcs::import_trades::CsvRecord;


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

/// Trade represents each unique buy/sell transaction. Created from a CsvRecord and Config 
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
    pub fn new(record: CsvRecord, config: &Config) -> Result<Self, Box<dyn Error>> {
        
        // NaiveDateTime interpolation
        let trade_time = parse_datetime_string(&record.timestamp)?;
        let unix_time: i64 = NaiveDateTime::timestamp(&trade_time);

        // TxnType interpolation
        let txn_type = TxnType::return_txn_type(
            &record.txn_type,
            &config.buy_txn_types,
            &config.sell_txn_types,
        );

        // Decimal conversions
        let base_asset_amount: Decimal = record.base_asset_amount.parse::<Decimal>().
            map_err(|e| ValueParseError::DecimalParseError { value: record.base_asset_amount.clone(), source: e })?;

        let quote_asset_amount: Decimal = record.quote_asset_amount.parse::<Decimal>().
            map_err(|e| ValueParseError::DecimalParseError { value: record.quote_asset_amount.clone(), source: e })?;
        
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
            base_asset: record.base_asset,
            base_asset_amount,
            quote_asset: record.quote_asset,
            quote_asset_amount,
            remaining,
            unix_time,
            price,
        })
    }
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
