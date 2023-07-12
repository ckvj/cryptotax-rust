use chrono::{NaiveDateTime, Datelike};
use cli_table::Table;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::cmp::Reverse;
use std::{cmp, collections::HashMap};
use itertools::Itertools;

use crate::funcs::config::{AccountingType, Config};
use crate::funcs::import_trades::{Asset, Trade, TxnType};


#[derive(Debug, Table)]
pub struct SaleEvent {
    #[table(title = "Asset Name")]
    name: String,
    #[table(title = "Buy Date")]
    buy_date: NaiveDateTime,
    #[table(title = "Buy Date (Unix)")]
    buy_date_unix: i64,
    #[table(title = "Sale Date")]
    sale_date: NaiveDateTime,
    #[table(title = "Sale Date (Unix)")]
    sale_date_unix: i64,
    #[table(title = "Purchase Price")]
    purchase_price: f32,
    #[table(title = "Sale Price")]
    sale_price: f32,
    #[table(title = "Amount")]
    amount: Decimal,
    #[table(title = "Gain-Loss")]
    pub gain_loss: f32,
    #[table(title = "Sell Year")]
    pub sell_year: i32,
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
        let amount = amount;
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

pub fn get_sale_events(trades: HashMap<String, Asset>, config: &Config) -> Vec<SaleEvent> {
    let mut sale_events: Vec<SaleEvent> = vec![];

    for (_, asset) in trades.iter() {
        let dust_threshold = Decimal::from_f32_retain(0.00001).unwrap(); // Account for deiminumus reporting errors
        let mut buy_txn_list = build_buy_list(&asset.trades, &config.accounting_type);
        let mut sale_txn_list = build_sale_list(&asset.trades);

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
                let event = SaleEvent::new(buy, sale, clip_size); // Why not &buy & &sell?

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


pub fn get_annual_summary(sales: &[SaleEvent]) -> HashMap<String,HashMap<i32,f32>> {
    
    let unique_names: Vec<String> = sales
        .iter()
        .map(|sale: &SaleEvent| sale.name.to_owned())
        .collect::<Vec<String>>()
        .into_iter()
        .unique()
        .collect();

    let empty_hashmap: Vec<HashMap<i32, f32>> = vec![HashMap::new(); unique_names.len()];
 
    let mut annual_summary = HashMap::from_iter(unique_names.iter().cloned().zip(empty_hashmap.iter().cloned()));

    for sale in sales.iter() {
        if annual_summary[&sale.name].contains_key(&sale.sell_year) {
            let v = annual_summary.get_mut(&sale.name).unwrap().get_mut(&sale.sell_year).unwrap();
            *v += sale.gain_loss; 
        } else {
            annual_summary.get_mut(&sale.name).unwrap().insert(sale.sell_year, sale.gain_loss);
        }
    }

    annual_summary
}

fn build_buy_list(trades: &[Trade], analysis_type: &AccountingType) -> Vec<Trade> {
    let mut buy_list: Vec<Trade> = trades
        .iter()
        .filter(|trade| trade.txn_type == TxnType::Buy)
        .cloned()
        .collect();

    match analysis_type {
        AccountingType::FIFO => buy_list.sort_by_key(|k| k.trade_time),
        AccountingType::LIFO => buy_list.sort_by_key(|k| Reverse(k.trade_time)),
        AccountingType::HIFO => buy_list.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap()),
    }
    buy_list
}

fn build_sale_list(trades: &[Trade]) -> Vec<Trade> {
    let mut sale_list: Vec<Trade> = trades
        .iter()
        .filter(|trade| trade.txn_type == TxnType::Sale)
        .cloned()
        .collect();
    
    sale_list.sort_by_key(|k| k.trade_time);
    sale_list
}
