use std::{collections::HashMap, cmp};

use chrono::NaiveDateTime;
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::{import_trades::{Trade, TxnType, Asset}, config::AccountingType};
use crate::config::Config;

use std::cmp::Reverse;


#[derive(Debug)]
pub struct SaleEvent {
    name: String,
    buy_date: NaiveDateTime,
    buy_date_unix: i64,
    sale_date: NaiveDateTime,
    sale_date_unix: i64,
    purchase_price: f32,
    sale_price: f32,
    amount: Decimal,
    pub gain_loss: f32,
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
        }
    }
}


pub fn get_sale_events(trades: HashMap<String, Asset>, config: &Config) -> Vec<SaleEvent> {

    let mut sale_events: Vec<SaleEvent> = vec![];

    for (_ , asset) in trades.iter() {

        let dust_threshold = Decimal::from_f32_retain(0.00001).unwrap();
        let mut buy_txn_list = build_buy_list(asset, &config.accounting_type);
        let mut sale_txn_list = build_sale_list(asset);

        if sale_txn_list.is_empty() {
            continue
        }

        for sale in sale_txn_list.iter_mut() {

            // Identify Next Buy
            for buy in buy_txn_list.iter_mut() {

                // Guard
                if buy.unix_time > sale.unix_time || buy.remaining < dust_threshold {
                    continue
                }

                let clip_size = cmp::min(buy.remaining, sale.remaining);
                let event = SaleEvent::new(&buy, &sale, clip_size);
                //dbg!(&event);
                sale_events.push(event);

                buy.remaining -= clip_size;
                sale.remaining -= clip_size;

                if buy.remaining < dust_threshold {
                    continue
                }
            }

            if sale.remaining < dust_threshold {
                continue;
            }
        }
    }
    sale_events
}

fn build_buy_list(asset: &Asset, analysis_type: &AccountingType) -> Vec<Trade> {

    let mut buy_list: Vec<Trade> = vec![];
    for trade in asset.trades.iter() {
        if trade.txn_type == TxnType::Buy {
            buy_list.push(trade.clone())
        }
    }
    
    match analysis_type {
        AccountingType::FIFO => buy_list.sort_by_key(|k| k.trade_time), 
        AccountingType::LIFO => buy_list.sort_by_key(|k| Reverse(k.trade_time)),
        AccountingType::HIFO => buy_list.sort_by(|a,b| b.price.partial_cmp(&a.price).unwrap()),
    }
    buy_list
}

fn build_sale_list(asset: &Asset) -> Vec<Trade> {

    let mut sale_list: Vec<Trade> = vec![];
    for trade in asset.trades.iter() {
        if trade.txn_type == TxnType::Sale {
            sale_list.push(trade.clone())
        }
    }
    sale_list.sort_by_key(|k| k.trade_time);
    sale_list
}