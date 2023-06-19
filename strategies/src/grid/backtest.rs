// main.rs
use binance::api::*;
use binance::market::*;
use std::f64::NAN;

// 计算网格位置
fn calculate_grid_positions(start_price: f64, end_price: f64, grid_num: usize) -> Vec<f64> {
    let interval = (end_price - start_price) / grid_num as f64;

    (0..=grid_num).map(|i| start_price + interval * i as f64).collect()
}

#[tokio::main]
async fn main() {
    let market: Market = Binance::new(None, None);
    let symbol = "BTCUSDT";
    let interval = "1d";

    let test_period = 500;

    let klines = market.get_klines(symbol, interval, test_period, None, None).await.unwrap();
    let closing_prices: Vec<f64> = klines.iter().map(|k| k.close).collect();

    let start_price = 30000.0;
    let end_price = 60000.0;
    let grid_num = 50;

    let grid_positions = calculate_grid_positions(start_price, end_price, grid_num);

    let mut balance = 100.0;
    let mut asset = 0.0;
    let buy_fee = 0.001;
    let sell_fee = 0.001;

    for close_price in closing_prices {
        if close_price.is_nan() {
            continue;
        }

        for i in 0..grid_positions.len() - 1 {
            let lower = grid_positions[i];
            let upper = grid_positions[i + 1];

            if close_price >= lower && close_price < upper {
                let buy_amount = balance / upper * (1.0 - buy_fee);
                println!("Buy {:.8} asset at {:.2} price", buy_amount, upper);
                balance = 0.0;
                asset += buy_amount;

                let sell_amount = asset * lower * (1.0 - sell_fee);
                println!("Sell {:.8} asset at {:.2} price", asset, lower);
                balance += sell_amount;
                asset = 0.0;

                break;
            }
        }
    }

    println!("Final balance after backtesting: {:.2}", balance + asset * closing_prices.last().unwrap());
}