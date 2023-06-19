// main.rs
use binance::api::*;
use binance::market::*;
use coi::Inject;
use dashmap::DashMap;
use std::iter::repeat_with;
use std::f64::NAN;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::StreamExt;

// 计算给定周期的简单移动平均
fn moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![NAN; prices.len()];
    }

    let mut sma = vec![NAN; period - 1];
    let c: Vec<f64> = prices[0..period - 1].to_vec();
    let mut sum = c.iter().map(|&x| x).sum::<f64>();

    for window in prices.windows(period) {
        sum += window[period - 1] - window[0];
        sma.push(sum / period as f64);
    }
    sma
}

// 计算数组的标准差
fn standard_deviation(prices: &[f64], mean_prices: &[f64]) -> Vec<f64> {
    prices.iter().zip(mean_prices.iter())
        .map(|(&price, &mean)| {
            if price.is_nan() || mean.is_nan() {
                NAN
            } else {
                (price - mean).powi(2)
            }
        }).collect()
}

#[tokio::main]
async fn main() {
    let market: Market = Binance::new(None, None);
    let symbol = "BTCUSDT";
    let limit = 20;

    let mut intervals = vec!["1m".to_string()];
    let mut cache_interval = DashMap::new();

    for interval in intervals.iter() {
        let klines = market.get_klines(symbol, interval, limit, None, None).await.unwrap();

        let closing_prices: Vec<f64> = klines.iter().map(|k| k.close).collect();
        let sma = moving_average(&closing_prices, limit);
        let std_dev = standard_deviation(&closing_prices, &sma);

        let upper_band: Vec<f64> = sma.iter().zip(std_dev.iter())
            .map(|(&sma, &std_dev)| if sma.is_nan() || std_dev.is_nan() { NAN } else { sma + 2.0 * std_dev }).collect();
        let lower_band: Vec<f64> = sma.iter().zip(std_dev.iter())
            .map(|(&sma, &std_dev)| if sma.is_nan() || std_dev.is_nan() { NAN } else { sma - 2.0 * std_dev }).collect();

        cache_interval.insert(interval.to_string(), (sma, upper_band, lower_band));
    }

    let arc_cache = Arc::new(RwLock::new(cache_interval));

    let cache_update = Arc::clone(&arc_cache);

    let worker = async move {
        loop {
            for interval in intervals.iter() {
                if let Some(cached_data) = cache_update.read().await.get(interval) {
                    let (sma, upper_band, lower_band) = cached_data.value();
                    println!("布林带 (周期: {}):\n中线: {:?}\n上轨: {:?}\n下轨: {:?}", interval, sma, upper_band, lower_band);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    };

    tokio::spawn(worker).await.unwrap();
}