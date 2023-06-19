use binance::api::*;
use binance::market::*;
use ta::{indicators::BollingerBands, Next};
use std::f64::NAN;

async fn fetch_closing_prices(market: &binance::market::Market, symbol: &str, interval: &str, limit: u32) -> Vec<f64> {
    let klines = market.get_klines(symbol, interval, limit, None, None).await.unwrap();
    klines.iter().map(|k| k.close).collect()
}

#[tokio::main]
async fn main() {
    let market: Market = Binance::new(None, None);
    let symbol = "BTCUSDT";
    let interval = "1d";
    let test_period = 500;

    let closing_prices = fetch_closing_prices(&market, symbol, interval, test_period).await;

    let mut bbands = BollingerBands::new(20, 2.0).unwrap();

    let mut balance = 100.0;
    let mut asset = 0.0;
    let buy_fee = 0.001;
    let sell_fee = 0.001;

    for close_price in &closing_prices {
        bbands.next(*close_price);

        let (lower, _, upper) = bbands.last();
        if let (Some(lower), Some(upper)) = (lower, upper) {
            if *close_price < lower && balance > 0.0 {
                let buy_amount = balance / *close_price * (1.0 - buy_fee);
                println!("Buy {:.8} asset at {:.2} price", buy_amount, *close_price);
                balance = 0.0;
                asset += buy_amount;
            } else if *close_price > upper && asset > 0.0 {
                let sell_amount = asset * *close_price * (1.0 - sell_fee);
                println!("Sell {:.8} asset at {:.2} price", asset, *close_price);
                balance += sell_amount;
                asset = 0.0;
            }
        }
    }

    let final_balance = balance + asset * closing_prices.last().unwrap_or(&NAN);
    println!("Final balance after backtesting: {:.2}", final_balance);
}