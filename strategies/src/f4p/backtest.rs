// main.rs
use binance::api::*;
use binance::market::*;
use std::f64::NAN;

async fn get_klines(market: &binance::market::Market, symbol: &str, interval: &str, limit: u32) -> Vec<binance::model::KlineSummary> {
    let klines = market.get_klines(symbol, interval, limit, None, None).await.unwrap();
    klines
}

fn filippi_4price(close: f64, open: f64, high: f64, low: f64) -> f64 {
    (close + open + high + low) / 4.0
}

#[tokio::main]
async fn main() {
    let market: Market = Binance::new(None, None);
    let symbol = "BTCUSDT";
    let interval = "1d";

    let buy_trigger = 0.01;
    let sell_trigger = 0.01;

    let klines = get_klines(&market, symbol, interval, 500).await;

    let mut balance = 100.0;
    let mut asset = 0.0;

    for (i, kline) in klines.iter().enumerate() {
        if i < 1 {
            continue;
        }

        let filippi = filippi_4price(kline.close, kline.open, kline.high, kline.low);

        let prev_kline = &klines[i - 1];

        if filippi * (1.0 + buy_trigger) < prev_kline.close && balance > 0.0 {
            let buy_amount = balance / prev_kline.close;
            println!("Day {}: Buy {:.8} asset at {:.2} price", i, buy_amount, prev_kline.close);

            balance = 0.0;
            asset += buy_amount;
        } else if filippi * (1.0 - sell_trigger) > prev_kline.close && asset > 0.0 {
            let sell_amount = asset * prev_kline.close;
            println!("Day {}: Sell {:.8} asset at {:.2} price", i, asset, prev_kline.close);

            balance += sell_amount;
            asset = 0.0;
        }
    }

    println!("Final balance after trading: {:.2}", balance + asset * klines.last().unwrap().close);
}