// use binance::{account::*, api::*, general::*, market::*};
// use binance::model::*;
// use binance::websockets::*;
// use tokio::task;
// use ta::indicators::AverageTrueRange;
// use ta::Next;
// use async_trait::async_trait;
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
// #[async_trait]
// impl Backtester for AtrBacktester {
//     async fn run(&self) -> Result<(), String> {
//         fn calculate_atr(klines: &[Kline], period: u32) -> Vec<f64> {
//             let mut atr = AverageTrueRange::new(period).unwrap();
//             let mut atr_values = Vec::new();
//
//             for i in 0..klines.len() {
//                 let high = klines[i].high.parse::<f64>().unwrap_or_default();
//                 let low = klines[i].low.parse::<f64>().unwrap_or_default();
//                 let close = if i == 0 {
//                     0.0
//                 } else {
//                     klines[i - 1].close.parse::<f64>().unwrap_or_default()
//                 };
//
//                 let tr = ta::high_low_close(high, low, close);
//
//                 let atr_value = atr.next(tr);
//                 atr_values.push(atr_value);
//             }
//
//             atr_values
//         }
//
//         async fn fetch_klines(symbol: &str, interval: &str) -> Result<Vec<Kline>, Error> {
//             let client = reqwest::Client::new();
//             let url = format!(
//                 "https://api.binance.com/api/v3/klines?symbol={}&interval={}",
//                 symbol, interval
//             );
//
//             let response = client.get(&url).send().await?.json::<Vec<Vec<String>>>().await?;
//
//             let klines: Vec<Kline> = response
//                 .into_iter()
//                 .map(|k| Kline {
//                     timestamp: k[0].parse().unwrap_or(0),
//                     open: k[1].clone(),
//                     high: k[2].clone(),
//                     low: k[3].clone(),
//                     close: k[4].clone(),
//                 })
//                 .collect();
//
//             Ok(klines)
//         }
//
//         let klines = fetch_klines(&self.trading_pair, &self.interval).await.unwrap();
//         let atr_values = calculate_atr(&klines, self.atr_period);
//
//         // Implement your ATR-based backtesting strategy here
//         for i in 0..atr_values.len() {
//             println!(
//                 "Timestamp: {} ATR: {}",
//                 klines[i].timestamp, atr_values[i]
//             );
//         }
//
//         Ok(())
//     }
// }
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let trading_pairs = vec!["BTCUSDT", "ETHUSDT"];
//     let interval = "1h";
//     let atr_period = 14;
//
//     let mut tasks = Vec::new();
//
//     for trading_pair in trading_pairs {
//         let backtester = AtrBacktester {
//             trading_pair: trading_pair.to_owned(),
//             interval: interval.to_owned(),
//             atr_period,
//         };
//
//         tasks.push(task::spawn(async move { backtester.run().await }));
//     }
//
//     for t in tasks {
//         t.await.unwrap();
//     }
//
//     Ok(())
// }