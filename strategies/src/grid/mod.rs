use binance::account::*;
use binance::api::*;
use binance::general::*;
use binance::market::*;
use binance::model::*;
use binance::websockets::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use futures::{StreamExt, future::join_all};

pub mod backtest;


struct GridTradingStrategy {
    trading_pair: String,
    grid_levels: usize,
    lower_bound: f64,
    upper_bound: f64,
}

async fn listen_fills(api_key: &str) -> Result<impl futures::Stream<Item = Result<AggTrade, String>>, String> {
    let agg_trade: String = format!("{}@aggTrade", "btcusdt");
    let mut web_socket: WebSockets = WebSockets::new();
    web_socket.add_socket(agg_trade, api_key)?;
    Ok(web_socket)
}

async fn start_grid_trading(api_key: &str, secret_key: &str) -> Result<(), String> {
    let api_key = Arc::from(api_key);
    let secret_key = Arc::from(secret_key);
    let trading_pair = "BTCUSDT".to_string();
    let capitalize = Arc::new(Mutex::new(HashSet::new()));

    // 初始化网格策略
    let strategy = GridTradingStrategy {
        trading_pair: trading_pair.clone(),
        grid_levels: 10,
        lower_bound: 30000.0,
        upper_bound: 40000.0,
    };

    let price_step = (strategy.upper_bound - strategy.lower_bound) / (strategy.grid_levels as f64);

    // 创建网格订单
    let mut order_tasks = Vec::new();
    for i in 0..strategy.grid_levels {
        let buy_price = strategy.lower_bound + (i as f64) * price_step;
        let sell_price = strategy.lower_bound + ((i + 1) as f64) * price_step;

        // 此处需根据交易对及资产量动态计算交易数量
        let quantity = 0.001;

        let api_key = api_key.clone();
        let secret_key = secret_key.clone();
        let trading_pair = trading_pair.clone();

        order_tasks.push(tokio::spawn(async move {
            create_limit_order(
                &api_key,
                &secret_key,
                &trading_pair,
                "BUY",
                buy_price,
                quantity,
            )
            .await
        }));

        let api_key = api_key.clone();
        let secret_key = secret_key.clone();
        let trading_pair = trading_pair.clone();

        order_tasks.push(tokio::spawn(async move {
            create_limit_order(
                &api_key,
                &secret_key,
                &trading_pair,
                "SELL",
                sell_price,
                quantity,
            )
            .await
        }));
    }

    let _order_results = join_all(order_tasks).await;

    // 监听最新的成交订单并调整策略
    let trading_pair_api_key = api_key.clone();
    let fill_stream = listen_fills(&trading_pair_api_key).await?;
    fill_stream.for_each(|agg_trade_result| async move {
        if let Ok(agg_trade) = agg_trade_result {
            let price: f64 = agg_trade.price.parse().unwrap_or_default();
            println!("Latest trade price: {}", price);

            // 通过监听已成交订单，根据实际情况调整网格策略，例如撤销未成交订单重新下单等
        }
    }).await;

    Ok(())
}

#[tokio::main]
async fn main() {
    let api_key = "YOUR_BINANCE_API_KEY";
    let secret_key = "YOUR_BINANCE_SECRET_KEY";

    match start_grid_trading(api_key, secret_key).await {
        Ok(_) => println!("网格交易已执行"),
        Err(error) => println!("执行失败：{}", error),
    }
}