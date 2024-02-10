/// Asset holds Trade events for a given asset

use crate::funcs::trade::Trade;

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub trades: Vec<Trade>,
}

