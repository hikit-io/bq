use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use strategies::{Data, DataCategory, DataId, DataIndex, Signal, Strategies, Strategy};
use tokio::sync::RwLock;

use crate::{
    channel::{Broadcast, Mpsc},
    DataChannelIndex,
};

// 策略生效模式
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrategyMode {
    #[default]
    Or,
    // And,
    // Weight,
}

#[derive(Debug)]
pub struct StrategySignal {
    id: u64,
    symbol: String,
    signal: Signal,
}

pub enum State {
    WaitBuy,
    WaitSell,
}

type EventTime = u64;

/// 一个实例只能拥有一个订单
pub struct Instance {
    pub(crate) symbol: String,                           // 交易对名称
    pub(crate) strategies: Vec<Arc<RwLock<Strategies>>>, // 策略
    pub(crate) strategy_mode: StrategyMode,              // 策略模式
    pub(crate) signal_channel: Mpsc<StrategySignal>,     // 接收来自策略的交易信号
    pub(crate) state: Arc<RwLock<State>>, // 实例状态，初始为wait buy，挂买单转wait sell，买单成功后挂卖单，卖单完成后转wait buy。
    pub(crate) current_signals: Arc<RwLock<HashMap<EventTime, Vec<StrategySignal>>>>, // 缓存信号
}

impl Instance {
    pub fn new(symbol: &str, mode: StrategyMode, strategies: Vec<Strategies>) -> Self {
        Self {
            symbol: symbol.to_string(),
            strategies: strategies
                .into_iter()
                .map(|e| Arc::new(RwLock::new(e)))
                .collect::<Vec<_>>(),
            strategy_mode: mode,
            signal_channel: Default::default(),
            state: Arc::new(RwLock::new(State::WaitBuy)),
            current_signals: Default::default(),
        }
    }

    pub async fn run(&mut self, data_channels: &HashMap<DataChannelIndex, Broadcast<Data>>) {
        self.run_strategies(data_channels).await;
        // 处理交易信号
        let mut signal_rx = unsafe { self.signal_channel.rx.take().unwrap_unchecked() };
        let current_signals = self.current_signals.clone();
        tokio::spawn({
            async move {
                loop {
                    if let Some(signal) = signal_rx.recv().await {
                        tracing::info!("Instance handle signal{:?}", signal);
                        let mut current_signals = current_signals.write().await;
                        if let Some(signals) = current_signals.get_mut(&signal.id) {
                            signals.push(signal);
                        }

                        // 检查每一轮
                    }
                }
            }
        });
    }

    pub async fn run_strategies(&self, data_channels: &HashMap<DataChannelIndex, Broadcast<Data>>) {
        for strategy in self.strategies.iter().map(Clone::clone) {
            let signal_tx = self.signal_channel.tx.clone();
            let data_category = { strategy.read().await.data_category() };

            let mut data_rx = data_channels
                .get(&(self.symbol.clone(), data_category).data_index())
                .unwrap()
                .tx
                .subscribe();

            tokio::spawn({
                let symbol = self.symbol.clone();
                async move {
                    loop {
                        match data_rx.recv().await {
                            Ok(data) => {
                                let data_id = data.data_id(); // 数据ID用于确认是否为同一刻数据。
                                let signal = { strategy.write().await.signal(data) };
                                let _ = signal_tx.send(StrategySignal {
                                    id: data_id,
                                    symbol: symbol.clone(),
                                    signal,
                                });
                            }
                            Err(err) => {
                                tracing::info!("Recv Data failed, {:?}", err);
                            }
                        }
                    }
                }
            });
        }
    }
}
