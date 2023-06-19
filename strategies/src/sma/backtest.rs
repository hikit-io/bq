// main.rs
use binance::api::*;
use binance::market::*;
use std::iter::repeat_with;
use std::f64::NAN;
use std::cmp::Ordering;

// 计算简单移动平均
fn sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![NAN; prices.len()];
    }

    let mut sma = vec![NAN; period - 1];
    let mut sum: f64 = prices[..(period - 1)].iter().copied().sum();
    for window in prices.windows(period) {
        sum += window[period - 1] - window[0];
        sma.push(sum / period as f64);
    }
    sma
}

#[tokio::main]
async fn main() {
    let market: Market = Binance::new(None, None);
    let symbol = "BTCUSDT";
    let interval = "1d";

    let sma_period = 10;
    let test_period = 500;

    let klines = market.get_klines(symbol, interval, test_period, None, None).await.unwrap();
    let closing_prices: Vec<f64> = klines.iter().map(|k| k.close).collect();
    let sma_values = sma(&closing_prices, sma_period);

    let mut balance = 100.0;
    let mut asset = 0.0;
    let buy_fee = 0.001;
    let sell_fee = 0.001;

    for i in sma_period..(test_period - 1) {
        let (last_close, last_sma) = (closing_prices[i], sma_values[i - 1]);
        let (prev_close, prev_sma) = (closing_prices[i - 1], sma_values[i - 2]);

        match (prev_close.partial_cmp(&prev_sma), last_close.partial_cmp(&last_sma)) {
            (Some(Ordering::Less), Some(Ordering::Greater)) if balance > 0.0 => {
                let buy_amount = balance / last_close * (1.0 - buy_fee);
                println!("Day {}: Buy {:.8} asset at {:.2} price", i, buy_amount, last_close);

                balance = 0.0;
                asset += buy_amount;
            }
            (Some(Ordering::Greater), Some(Ordering::Less)) if asset > 0.0 => {
                let sell_amount = asset * last_close * (1.0 - sell_fee);
                println!("Day {}: Sell {:.8} asset at {:.2} price", i, asset, last_close);

                balance += sell_amount;
                asset = 0.0;
            }
            _ => {}
        }
    }

    println!("Final balance after backtesting: {:.2}", balance + asset * closing_prices.last().unwrap());
}