use std::env;
use binance::api::Binance;
use ta::indicators::{ExponentialMovingAverage, MovingAverageConvergenceDivergence};

async fn fetch_macd_history(api_key: &str, secret_key: &str, interval: KlineInterval) -> ()) {
    let binance = Binance::with_credential(api_key, secret_key);

    // Fetch historical klines data
    let historical_klines = binance.get_klines_history("BTCUSDT", interval, None, None, None).await.unwrap();

    let mut close_prices: Vec<f64> = historical_klines.into_iter().map(|k| k.close).collect();

    // Calculate the MACD
    let ema_short = ExponentialMovingAverage::new(12).unwrap();
    let ema_long = ExponentialMovingAverage::new(26).unwrap();
    let signal = ExponentialMovingAverage::new(9).unwrap();
    let mut macd = MovingAverageConvergenceDivergence::new(ema_short, ema_long, signal).unwrap();

    for price in close_prices.iter() {
        macd.next(*price);
    }

    println!("Current MACD: {:?}", macd);
}

#[tokio::main]
async fn main() {
    // Load Binance API keys from environment variables
    let api_key = env::var("BINANCE_API_KEY").expect("BINANCE_API_KEY must be set");
    let secret_key = env::var("BINANCE_SECRET_KEY").expect("BINANCE_API_SECRET must be set");

    fetch_macd_history(&api_key, &secret_key, KlineInterval::FourHour).await;
}