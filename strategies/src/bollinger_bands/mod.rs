// main.rs
use binance::api::*;
use binance::account::*;
use binance::market::*;
use ta::indicators::{BollingerBands, Indicator};
use std::f64::NAN;
use ta::Next;

pub mod backtest;

// 设置API的key和secret.
const API_KEY: &str = "YOUR_API_KEY";
const API_SECRET: &str = "YOUR_API_SECRET";

async fn fetch_closing_prices(market: &binance::market::Market, symbol: &str, interval: &str) -> Vec<f64> {
    let klines = market.get_klines(symbol, interval, 50, None, None).await.unwrap();
    klines.iter().map(|k| k.close).collect()
}

#[tokio::main]
async fn main() {
    let market: Market = Binance::new(None, None);
    let account: Account = Binance::new(Some(API_KEY.to_string()), Some(API_SECRET.to_string()));
    let symbol = "BTCUSDT";
    let interval = "1h";

    let mut bbands = BollingerBands::new(20, 2.0).unwrap();

    loop {
        let closing_prices = fetch_closing_prices(&market, symbol, interval).await;

        for close_price in &closing_prices {
            bbands.next(*close_price);
        }

        let (lower, _, upper) = bbands.last();

        if let (Some(lower), Some(upper)) = (lower, upper) {
            let last_price = closing_prices.last().unwrap_or(&NAN).to_owned();

            if last_price < lower {
                println!("Buy signal detected");
                // Define your buy order logic here
            } else if last_price > upper {
                println!("Sell signal detected");
                // Define your sell order logic here
            } else {
                println!("Within Bollinger Bands");
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60)).await;
    }
}