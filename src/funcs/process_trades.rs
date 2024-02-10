use chrono::{Datelike, NaiveDateTime};
use cli_table::Table;
use itertools::Itertools;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::cmp::Reverse;
use std::{cmp, collections::HashMap};

use crate::funcs::config::{AccountingType, Config};
use crate::funcs::trade::{Asset, Trade};
use crate::funcs::txn_type::TxnType;


/// SaleEvent is a Struct that holds individual sale event
#[derive(Debug, Table, Clone)]
pub struct SaleEvent {
    #[table(title = "Asset Name")] name: String,
    #[table(title = "Buy Date")] buy_date: NaiveDateTime,
    #[table(title = "Buy Date (Unix)")] buy_date_unix: i64,
    // #[table(title = "Buy Venue")] buy_venue: String,
    #[table(title = "Sale Date")] sale_date: NaiveDateTime,
    #[table(title = "Sale Date (Unix)")] sale_date_unix: i64,
    // #[table(title = "Sale Venue")] sale_venue: String,
    #[table(title = "Purchase Price")] purchase_price: f32,
    #[table(title = "Sale Price")] sale_price: f32,
    #[table(title = "Amount")] amount: Decimal,
    #[table(title = "Gain-Loss")] pub gain_loss: f32,
    #[table(title = "Sell Year")] pub sell_year: i32,
}

impl SaleEvent {
    fn new(buy: &Trade, sale: &Trade, amount: Decimal) -> Self {
        let name: String = sale.base_asset.to_owned();
        let buy_date: NaiveDateTime = buy.trade_time.to_owned();
        let buy_date_unix: i64 = buy.unix_time.to_owned();
        // let buy_venue: String = buy.venue.to_owned().unwrap_or("".to_string());
        let sale_date = sale.trade_time.to_owned();
        let sale_date_unix: i64 = sale.unix_time.to_owned();
        // let sale_venue: String = sale.venue.to_owned().unwrap_or("".to_string());
        let purchase_price: f32 = buy.price.to_owned();
        let sale_price = sale.price.to_owned();
        let gain_loss = (sale_price - purchase_price) * amount.to_f32().unwrap();
        let sell_year = sale.trade_time.year();

        Self {
            name,
            buy_date,
            buy_date_unix,
            // buy_venue,
            sale_date,
            sale_date_unix,
            // sale_venue,
            purchase_price,
            sale_price,
            amount,
            gain_loss,
            sell_year,
        }
    }
}



/// Gets the sale events for an asset.
///
/// # Arguments
///
/// * `trades` - A hash map of trades, which are asset names & Asset structs containting trades
/// * `config` - The configuration object from the config ini file.
///
/// # Returns
///
/// A list of sale events for the asset.
pub fn get_sale_events(trades: HashMap<String, Asset>, config: &Config) -> Vec<SaleEvent> {
    let mut sale_events: Vec<SaleEvent> = vec![];
    let dust_threshold = Decimal::from_f32_retain(0.00001).unwrap(); // Account for deiminumus reporting errors


    for (_, asset) in trades.iter() {

        
        let mut buy_txn_list = build_buy_list(&asset.trades, config);
        let mut sale_txn_list = build_sale_list(&asset.trades, config);

        if sale_txn_list.is_empty() {
            continue;
        }

        for sale in sale_txn_list.iter_mut() {
            for buy in buy_txn_list.iter_mut() {
                // Proceed for valid buy events

                if buy.unix_time > sale.unix_time || buy.remaining < dust_threshold {
                    continue;
                }

                let clip_size = cmp::min(buy.remaining, sale.remaining);
                let event = SaleEvent::new(buy, sale, clip_size);

                sale_events.push(event);

                buy.remaining -= clip_size;
                sale.remaining -= clip_size;

                if buy.remaining < dust_threshold {
                    continue;
                }
            }

            if sale.remaining < dust_threshold {
                continue;
            }
        }
    }
    sale_events
}


/// Gets the annual summary of sales.
///
/// # Arguments
///
/// * `sales` - A list of sale events.
///
/// # Returns
///
/// A hash map where the keys are the asset names and the values are hash maps
/// where the keys are the sell years and the values are the gain or loss for that
/// asset in that year.
pub fn get_annual_summary(sales: &[SaleEvent]) -> HashMap<String, HashMap<i32, f32>> {
    
    let unique_assets: Vec<String> = sales
        .iter()
        .map(|sale: &SaleEvent| sale.name.to_owned())
        .collect::<Vec<String>>()
        .into_iter()
        .unique()
        .collect();

    let empty_hashmap: Vec<HashMap<i32, f32>> = vec![HashMap::new(); unique_assets.len()];

    let mut annual_summary = HashMap::from_iter(
        unique_assets
            .iter()
            .cloned()
            .zip(empty_hashmap.iter().cloned()),
    );

    for sale in sales.iter() {
        if annual_summary[&sale.name].contains_key(&sale.sell_year) {
            let v = annual_summary
                .get_mut(&sale.name)
                .unwrap()
                .get_mut(&sale.sell_year)
                .unwrap();
            *v += sale.gain_loss; // Question: Why do I need to dereference here?
        } else {
            annual_summary
                .get_mut(&sale.name)
                .unwrap()
                .insert(sale.sell_year, sale.gain_loss);
        }
    }

    annual_summary
}



/// Builds a list of buys from the given list of trades, sorted by the specified accounting type.
///
/// # Arguments
///
/// * `trades` - The list of trades to build the buy list from.
/// * `analysis_type` - The accounting type to sort the buy list by.
///
/// # Returns
///
/// A list of buys, sorted by the specified accounting type.
///
pub fn build_buy_list(trades: &[Trade], config: &Config) -> Vec<Trade> {
    let mut buy_list: Vec<Trade> = trades
        .iter()
        .filter(|trade| trade.txn_type == TxnType::Buy)
        .cloned()
        .collect();

    // if config.venues.is_some() {
    //     let venues = config.venues.clone().unwrap();
    //     buy_list = retain_only_designated_venues(&buy_list, venues);
    // }

    match config.accounting_type {
        AccountingType::FIFO => buy_list.sort_by_key(|k| k.trade_time),
        AccountingType::LIFO => buy_list.sort_by_key(|k| Reverse(k.trade_time)),
        AccountingType::HIFO => buy_list.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap()),
    }
    buy_list
}



/// Builds a list of sales from the given list of trades.
///
/// # Arguments
///
/// * `trades` - The list of trades to build the sale list from.
///
/// # Returns
///
/// A list of sales, sorted by trade time.
fn build_sale_list(trades: &[Trade], config: &Config) -> Vec<Trade> {
    let mut sale_list: Vec<Trade> = trades
        .iter()
        .filter(|trade| trade.txn_type == TxnType::Sale)
        .cloned()
        .collect();

    // if config.venues.is_some() {
    //     let venues = config.venues.clone().unwrap();
    //     sale_list = retain_only_designated_venues(&sale_list, venues);
    // }

    sale_list.sort_by_key(|k| k.trade_time);
    sale_list
}


fn retain_only_designated_venues(trades: &Vec<Trade>, venues: Vec<String>) -> Vec<Trade> {
    let mut updated_trades = trades.to_owned();
    // updated_trades.retain(|trade| venues.contains(trade.venue.as_ref().unwrap()));

    updated_trades


} 

