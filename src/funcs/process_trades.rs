use chrono::{Datelike, NaiveDateTime};
use cli_table::Table;
use itertools::Itertools;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::{cmp, collections::HashMap};
use serde::{Serialize, Serializer};


use crate::funcs::config::{AccountingType, Config};
use crate::funcs::trade::Trade;
use crate::funcs::txn_type::TxnType;

use polars::prelude::*;

/// SaleEvent is a Struct that holds individual sale event
#[derive(Debug, Table, Clone, Serialize)]
pub struct SaleEvent {
    name: String,
    #[serde(serialize_with = "serialize_datetime")]
    buy_date: NaiveDateTime,
    buy_date_unix: i64,
    #[serde(serialize_with = "serialize_datetime")]
    sale_date: NaiveDateTime,
    sale_date_unix: i64,
    purchase_price: f32,
    sale_price: f32,
    amount: Decimal,
    pub gain_loss: f32,
    pub sell_year: i32,
}

fn serialize_datetime<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = dt.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&s)
}


impl SaleEvent {
    fn new(buy: &Trade, sale: &Trade, amount: Decimal) -> Self {
        let name: String = sale.base_asset.to_owned();
        let buy_date: NaiveDateTime = buy.trade_time.to_owned();
        let buy_date_unix: i64 = buy.unix_time.to_owned();
        let sale_date = sale.trade_time.to_owned();
        let sale_date_unix: i64 = sale.unix_time.to_owned();
        let purchase_price: f32 = buy.price.to_owned();
        let sale_price = sale.price.to_owned();
        let gain_loss = (sale_price - purchase_price) * amount.to_f32().unwrap();
        let sell_year = sale.trade_time.year();

        Self {
            name,
            buy_date,
            buy_date_unix,
            sale_date,
            sale_date_unix,
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
pub fn get_sale_events(all_trades: HashMap<String, Vec<Trade>>, config: &Config) -> Vec<SaleEvent> {
    let mut sale_events: Vec<SaleEvent> = vec![];
    let dust_threshold = Decimal::from_f32_retain(0.00001).unwrap(); // Account for deiminumus reporting errors


    for (_, trades) in all_trades.iter() {

        let mut buy_txn_list = build_buy_list(trades, &config.accounting_type); // Filter and sort buys based on accounting type (eg FIFO)
        let mut sale_txn_list = build_sale_list(trades); // Filter and sort sales chronologically

        if sale_txn_list.is_empty() {
            continue; // Skip asset if no sales
        }

        for sale in sale_txn_list.iter_mut() {
            for buy in buy_txn_list.iter_mut() {

                // Filter for invalid buy transactions
                if buy.unix_time > sale.unix_time || buy.remaining < dust_threshold {
                    continue;
                }

                let clip_size = cmp::min(buy.remaining, sale.remaining);
                let event = SaleEvent::new(buy, sale, clip_size);

                sale_events.push(event);

                buy.remaining -= clip_size;
                sale.remaining -= clip_size;

                if sale.remaining < dust_threshold {
                    break;
                }
                
                if buy.remaining < dust_threshold {
                    continue;
                }
            }
        }
    }
    sale_events
}



pub fn get_annual_summary(sales: &[SaleEvent]) -> DataFrame {

    let unique_assets: Vec<String> = sales
        .iter()
        .map(|sale: &SaleEvent| sale.name.to_owned())
        .collect::<Vec<String>>()
        .into_iter()
        .unique()
        .sorted()
        .collect();

    let unique_years: Vec<i32> = sales
        .iter()
        .map(|sale: &SaleEvent| sale.sell_year)
        .collect::<Vec<i32>>()
        .into_iter()
        .unique()
        .sorted()
        .collect();

    // Empty Map of <Year<Asset, Gain-Loss>>
    let mut map: BTreeMap<i32, BTreeMap<String, f32>> = BTreeMap::default();

    // Create a default inner Map for each year
    let default_empty_dict: BTreeMap<String, f32> = unique_assets
        .iter()
        .map(|asset| (asset.to_owned(), 0.0))
        .collect();

    for year in unique_years {
        map.insert(year, default_empty_dict.clone());
    }

    // Update gain-loss for each sale
    for sale in sales.iter() {
        let val = map.get_mut(&sale.sell_year).unwrap().get_mut(&sale.name).unwrap();
        *val += sale.gain_loss;
    }

    // Initialize Dataframe with Unique Assets
    let asset_series = Series::new("Asset Name", unique_assets);
    let mut df = DataFrame::new(vec![asset_series]).unwrap();

    // Loop over years to populate data
    for (k,v) in map {
        let _series = Series::new(&k.to_string(), v.values().cloned().collect::<Vec<f32>>());
        df.with_column(_series).unwrap();
    }
    df


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
pub fn build_buy_list(trades: &[Trade], acct_type: &AccountingType) -> Vec<Trade> {
    let mut buy_list: Vec<Trade> = trades
        .iter()
        .filter(|trade| trade.txn_type == TxnType::Buy)
        .cloned()
        .collect();

    match acct_type {
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
fn build_sale_list(trades: &[Trade]) -> Vec<Trade> {
    let mut sale_list: Vec<Trade> = trades
        .iter()
        .filter(|trade| trade.txn_type == TxnType::Sale)
        .cloned()
        .collect();

    sale_list.sort_by_key(|k| k.trade_time);
    sale_list
}


// pub fn convert_vec_to_df(sale_events: &[SaleEvent]) -> DataFrame {

//     let name: Series = Series::new("Asset Name", sale_events.iter().map(|event| event.name.clone()).collect::<Vec<String>>());
//     let buy_date: Series = Series::new("Buy Date", sale_events.iter().map(|event| event.buy_date).collect::<Vec<NaiveDateTime>>());
//     let buy_date_unix: Series = Series::new("Buy Date (Unix)", sale_events.iter().map(|event| event.buy_date_unix).collect::<Vec<i64>>());
//     let sale_date: Series = Series::new("Sale Date", sale_events.iter().map(|event| event.sale_date).collect::<Vec<NaiveDateTime>>());
//     let sale_date_unix: Series = Series::new("Sale Date (Unix)", sale_events.iter().map(|event| event.sale_date_unix).collect::<Vec<i64>>());
//     let purchase_price: Series = Series::new("Purchase Price", sale_events.iter().map(|event| event.purchase_price).collect::<Vec<f32>>());
//     let sale_price: Series = Series::new("Sale Price", sale_events.iter().map(|event| event.sale_price).collect::<Vec<f32>>());
//     let amount: Series = Series::new("Amount", sale_events.iter().map(|event| event.amount.to_f32().unwrap()).collect::<Vec<f32>>());
//     let gain_loss: Series = Series::new("Gain-Loss", sale_events.iter().map(|event| event.gain_loss).collect::<Vec<f32>>());
//     let sell_year: Series = Series::new("Sell Year", sale_events.iter().map(|event| event.sell_year).collect::<Vec<i32>>());

//     let df = DataFrame::new(vec![
//         name,
//         buy_date,
//         buy_date_unix,
//         sale_date,
//         sale_date_unix,
//         purchase_price,
//         sale_price,
//         amount,
//         gain_loss,
//         sell_year,
//     ]);

//     df.unwrap()
// }



// /// Gets the annual summary of sales.
// ///
// /// # Arguments
// ///
// /// * `sales` - A list of sale events.
// ///
// /// # Returns
// ///
// /// A hash map where the keys are the asset names and the values are hash maps
// /// where the keys are the sell years and the values are the gain or loss for that
// /// asset in that year.
// pub fn get_annual_summary(sales: &[SaleEvent]) -> HashMap<String, HashMap<i32, f32>> {

//     let unique_assets: Vec<String> = sales
//         .iter()
//         .map(|sale: &SaleEvent| sale.name.to_owned())
//         .collect::<Vec<String>>()
//         .into_iter()
//         .unique()
//         .collect();

//     let empty_hashmap: Vec<HashMap<i32, f32>> = vec![HashMap::new(); unique_assets.len()];

//     let mut annual_summary = HashMap::from_iter(
//         unique_assets
//             .iter()
//             .cloned()
//             .zip(empty_hashmap.iter().cloned()),
//     );

//     for sale in sales.iter() {
//         if annual_summary[&sale.name].contains_key(&sale.sell_year) {
//             let v = annual_summary
//                 .get_mut(&sale.name)
//                 .unwrap()
//                 .get_mut(&sale.sell_year)
//                 .unwrap();
//             *v += sale.gain_loss; // Question: Why do I need to dereference here?
//         } else {
//             annual_summary
//                 .get_mut(&sale.name)
//                 .unwrap()
//                 .insert(sale.sell_year, sale.gain_loss);
//         }
//     }

//     annual_summary
// }


// pub struct SaleEvent {
//     #[table(title = "Asset Name")] name: String,
//     #[table(title = "Buy Date")] buy_date: NaiveDateTime,
//     #[table(title = "Buy Date (Unix)")] buy_date_unix: i64,
//     #[table(title = "Sale Date")] sale_date: NaiveDateTime,
//     #[table(title = "Sale Date (Unix)")] sale_date_unix: i64,
//     #[table(title = "Purchase Price")] purchase_price: f32,
//     #[table(title = "Sale Price")] sale_price: f32,
//     #[table(title = "Amount")] amount: Decimal,
//     #[table(title = "Gain-Loss")] pub gain_loss: f32,
//     #[table(title = "Sell Year")] pub sell_year: i32,
// }
