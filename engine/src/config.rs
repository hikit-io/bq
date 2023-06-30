use serde::{Deserialize, Serialize};
use strategies::KlineInterval;

use crate::instance::StrategyMode;

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "principal")]
    pub principal: f64,

    //    #[serde(rename = "data_stream")]
    //    pub data_stream: DataStream,
    #[serde(rename = "instances")]
    pub instances: Vec<Instance>,
}
//
//#[derive(Serialize, Deserialize)]
//pub struct DataStream {
//    #[serde(rename = "kline")]
//    pub kline: Kline,
//}
//
//#[derive(Serialize, Deserialize)]
//pub struct Kline {
//    #[serde(rename = "interval")]
//    pub interval: Vec<Interval>,
//}

#[derive(Serialize, Deserialize)]
pub struct Instance {
    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "mode")]
    pub mode: StrategyMode,

    #[serde(rename = "strategies")]
    pub strategies: Vec<Strategy>,

    #[serde(rename = "stop_loss")]
    pub stop_loss: f64,

    #[serde(rename = "principal")]
    pub principal: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Strategy {
    Rsi {
        #[serde(rename = "type")]
        strategy_type: StrategyType,

        #[serde(rename = "interval")]
        interval: KlineInterval,

        #[serde(rename = "period")]
        period: u64,

        #[serde(rename = "buy_threshold")]
        buy_threshold: f64,

        #[serde(rename = "sell_threshold")]
        sell_threshold: f64,
    },
    Atr {
        #[serde(rename = "type")]
        strategy_type: StrategyType,

        #[serde(rename = "interval")]
        interval: KlineInterval,

        #[serde(rename = "period")]
        period: u64,

        #[serde(rename = "threshold")]
        threshold: f64,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrategyType {
    RSI,
    ATR,
}
