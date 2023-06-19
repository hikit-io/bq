// use binance::{account::*, api::*, general::*, market::*};
// use binance::model::*;
// use binance::websockets::*;
// use tokio::task;
// use ta::indicators::AverageTrueRange;
// use ta::Next;
// use async_trait::async_trait;
// use reqwest::Error;
// use serde::Deserialize;
//
// #[async_trait]
// pub trait Backtester {
//     async fn run(&self) -> Result<(), String>;
// }
//
// #[derive(Deserialize, Debug)]
// pub struct Kline {
//     pub timestamp: i64,
//     pub open: String,
//     pub high: String,
//     pub low: String,
//     pub close: String,
// }
//
// struct AtrBacktester {
//     trading_pair: String,
//     interval: String,
//     atr_period: u32,
// }
//
