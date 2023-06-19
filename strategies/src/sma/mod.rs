// main.rs
use binance::api::*;
use binance::account::*;
use binance::market::*;
use std::f64::NAN;

pub mod backtest;

async fn get_sma(market: &binance::market::Market, symbol: &str, interval: &str, period: usize) -> Vec<f64> {
    let klines = market.get_klines(symbol, interval, period as u32, None, None).await.unwrap();

    let prices: Vec<f64> = klines.iter().map(|k| k.close).collect();

    if prices.len() < period {
        return vec![NAN; prices.len()];
    }

    let mut sma = vec![NAN; period - 1];
    let mut sum: f64 = prices[..(period - 1)].iter().sum();

    for window in prices.windows(period) {
        sum += window[period - 1] - window[0];
        sma.push(sum / period as f64);
    }

    sma
}

#[tokio::main]
async fn main() {
    let market: Market = Binance::new(None, None);
    let account: Account = Binance::new(Some("YOUR_API_KEY".to_string()), Some("YOUR_API_SECRET".to_string()));
    let symbol = "BTCUSDT";
    let interval = "1d";
    let sma_short_period = 10;
    let sma_long_period = 20;

    loop {
        let sma_short = get_sma(&market, symbol, interval, sma_short_period).await;
        let sma_long = get_sma(&market, symbol, interval, sma_long_period).await;
        
        if sma_short.is_empty() || sma_long.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            continue;
        }

        let balance = account.get_account().await.expect("Failed to query account data!");
        let asset = balance.get_asset(symbol).expect("Failed to find asset in account!");
        let free = asset.free.parse::<f64>().expect("Failed to parse free asset amount!");

        // Buy signal - Short SMA crossed above Long SMA
        if sma_short.last() > sma_long.last() && free > 0.0 {
            println!("Buy signal detected");
            // Define your buy order logic here
        }

        // Sell signal - Short SMA crossed below Long SMA
        if sma_short.last() < sma_long.last() && free <= 0.0 {
            println!("Sell signal detected");
            // Define your sell order logic here
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}