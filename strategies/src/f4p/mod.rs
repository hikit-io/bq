// main.rs
use binance::api::*;
use binance::account::*;
use binance::market::*;
use std::f64::NAN;
use futures::StreamExt;

pub mod backtest;

// 设置API的key和secret.
const API_KEY: &str = "YOUR_API_KEY";
const API_SECRET: &str = "YOUR_API_SECRET";

async fn get_filippi_4price(market: &binance::market::Market, symbol: &str, interval: &str) -> Option<f64> {
    let klines = market.get_klines(symbol, interval, 1, None, None).await.ok()?;

    let kline = klines.first()?;
    let filippi = (kline.open + kline.close + kline.high + kline.low) / 4.0;

    Some(filippi)
}

#[tokio::main]
async fn main() {
    let market: Market = Binance::new(None, None);
    let account: Account = Binance::new(Some(API_KEY.to_string()), Some(API_SECRET.to_string()));
    let symbol = "BTCUSDT";
    let interval = "1d";

    loop {
        let filippi_4price = get_filippi_4price(&market, symbol, interval).await.unwrap_or(NAN);
        if filippi_4price.is_nan() {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            continue;
        }

        let balance = account.get_account().await.expect("Failed to query account data!");
        let asset = balance.get_asset(symbol).expect("Failed to find asset in account!");
        let free = asset.free.parse::<f64>().expect("Failed to parse free asset amount!");

        // 以菲阿里四价为标准，设置价格涨幅偏差作为买卖条件的触发因素
        let buy_trigger = 0.01;
        let sell_trigger = 0.01;

        if filippi_4price * (1.0 + buy_trigger) < kline.close && free > 0.0 {
            println!("Buy signal detected");
            // 在此处实现购买逻辑
        } else if filippi_4price * (1.0 - sell_trigger) > kline.close && free <= 0.0 {
            println!("Sell signal detected");
            // 在此处实现出售逻辑
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}