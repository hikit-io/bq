use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "symbols")]
    symbols: Vec<Symbol>,
    #[serde(rename = "data_stream")]
    data_stream: DataStream,
}

#[derive(Serialize, Deserialize)]
pub struct Symbol {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "mode")]
    mode: String,

    #[serde(rename = "strategies")]
    strategies: Strategies,
}

#[derive(Serialize, Deserialize)]
pub struct Strategies {
    #[serde(rename = "rsi")]
    rsi: Atr,

    #[serde(rename = "atr")]
    atr: Atr,
}

#[derive(Serialize, Deserialize)]
pub struct Atr {
    #[serde(rename = "peroid")]
    peroid: i64,
}

#[derive(Serialize, Deserialize)]
pub struct DataStream {
    kline: Kline,
}

#[derive(Serialize, Deserialize)]
pub struct Kline {
    interval: String,
}
