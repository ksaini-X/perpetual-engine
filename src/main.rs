use std::sync::Arc;

use futures_util::lock::Mutex;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::engine::registry::Registry;

pub mod engine;
pub mod market;

#[derive(Deserialize, Clone, Debug)]
pub enum Event {
    PriceUpdate(Decimal),
}

#[tokio::main]
async fn main() {
    let registry = Arc::new(Mutex::new(Registry::new()));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(100);

    let tx_for_feed = tx.clone();
}
