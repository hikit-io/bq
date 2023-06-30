use std::{
    any::Any,
    collections::{HashMap, HashSet},
    future::Future,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use action::Handle;
use binance::{
    account::Account,
    api::Binance,
    userstream::UserStream,
    websockets::{kline_stream, WebSockets},
    ws_model::{CombinedStreamEvent, OrderUpdate, WebsocketEvent, WebsocketEventUntag},
};
use channel::Mpsc;
use config::Config;
use instance::Instance;
use strategies::{
    atr::AverageTrueRange, rsi::RelativeStrengthIndex, Category, Data, DataCategory, DataId,
    DataIndex, Index, Signal, Strategies, Strategy,
};
use tokio::{
    sync::{broadcast, RwLock},
    time,
};

use crate::channel::Broadcast;

mod account;
mod action;
mod channel;
pub mod config;
mod instance;

type Symbol = String;
type OrderId = String;
type DataChannelIndex = Index<Symbol>;

pub struct Engine {
    account: Account,
    user_stream: UserStream,

    state: State,

    data_channels: HashMap<DataChannelIndex, Broadcast<Data>>,
    order_channel: Mpsc<OrderUpdate>,
    wss: Option<WebSockets<'static, CombinedStreamEvent<WebsocketEventUntag>>>,
    wss_streams: Vec<String>,

    decision: Mpsc<(Symbol, Signal)>,

    principal: f64,
}

pub(crate) struct Order {
    symbol: String,
    id: u64,

    quality: f64,
    buy_price: f64,
    min_sell_price: f64,

    status: OrderStatus,

    update_ts: u64,
}

pub(crate) enum OrderStatus {
    Committed,
    Accepted,
    Success,
    Canceled,
}

pub struct State {
    instances: Vec<Instance>,
    optimal_price: Arc<RwLock<HashMap<Symbol, Price>>>,
    profit: Arc<RwLock<f64>>,
    principal: Arc<RwLock<f64>>,
    orders: Arc<RwLock<HashMap<OrderId, Order>>>,
    archived_orders: Arc<RwLock<Vec<Order>>>,
}

pub struct Price {
    buy: f64,
    sell: f64,
}

impl Engine {
    pub fn new_with_env(config: Config) -> Self {
        let api_key = std::env::var("BQ_KEY").expect("read env var");
        let secret_key = std::env::var("BQ_SECRET").expect("read env var");
        Self::new(&api_key, &secret_key, config)
    }

    pub fn new(api_key: &str, secret_key: &str, config: Config) -> Self {
        tracing::info!(api_key, secret_key);
        // 初始化账户
        let account = Account::new(Some(api_key.to_string()), Some(secret_key.to_string()));
        let user_stream = UserStream::new(Some(api_key.to_string()), Some(secret_key.to_string()));

        // 数据通道
        let mut data_channels: HashMap<DataChannelIndex, Broadcast<Data>> = HashMap::new();
        let mut symbols = HashSet::new();
        let mut streams = HashSet::new();
        let mut instances = Vec::new();
        let mut order_channel = Mpsc::default();

        for inst_conf in config.instances {
            let mut strategies = Vec::new();
            for strategy in inst_conf.strategies {
                let (strategy, interval) = match strategy {
                    config::Strategy::Rsi {
                        strategy_type,
                        interval,
                        period,
                        buy_threshold,
                        sell_threshold,
                    } => (
                        Strategies::RelativeStrengthIndex(RelativeStrengthIndex::new(
                            period,
                            interval.clone(),
                            buy_threshold,
                            sell_threshold,
                        )),
                        interval,
                    ),
                    config::Strategy::Atr {
                        strategy_type,
                        interval,
                        period,
                        threshold,
                    } => (
                        Strategies::AverageTrueRange(AverageTrueRange::new(
                            period,
                            interval.clone(),
                        )),
                        interval,
                    ),
                };
                data_channels.insert(
                    (inst_conf.symbol.to_uppercase(), &strategy).data_index(),
                    Broadcast::<Data>::default(),
                );

                strategies.push(strategy);
                streams.insert(kline_stream(&inst_conf.symbol, &interval.to_string()));
            }

            // streams.insert(book_ticker_stream(&inst_conf.symbol));

            instances.push(Instance::new(
                &inst_conf.symbol.to_uppercase(),
                inst_conf.mode.clone(),
                strategies,
            ));

            data_channels.insert(
                (inst_conf.symbol.to_uppercase(), Category::BookTicker).data_index(),
                Broadcast::<Data>::default(),
            );

            symbols.insert(inst_conf.symbol.to_uppercase());
        }

        let data_channels_cloned = data_channels
            .iter()
            .map(|e| (e.0.clone(), e.1.clone()))
            .collect::<HashMap<_, _>>();

        let order_tx = order_channel.tx.clone();

        let wss = WebSockets::new(move |e: CombinedStreamEvent<WebsocketEventUntag>| {
            match e.data {
                WebsocketEventUntag::WebsocketEvent(WebsocketEvent::OrderUpdate(order)) => {
                    match order_tx.send(*order) {
                        Ok(()) => {
                            tracing::info!("Send OrderUpdate Ok");
                        }
                        Err(e) => {
                            tracing::error!("Send OrderUpdate failed: {:?}", e);
                        }
                    }
                }
                WebsocketEventUntag::WebsocketEvent(WebsocketEvent::Kline(kline)) => {
                    let mut data_tx = data_channels_cloned.get(&kline.data_index()).unwrap();
                    match data_tx.tx.send(Data::Kline(*kline)) {
                        Ok(size) => {
                            tracing::info!("Send Kline, size: {:?}", size);
                        }
                        Err(e) => {
                            tracing::error!("Send Kline failed,{:?}", e);
                        }
                    }
                }
                WebsocketEventUntag::BookTicker(bt) => {
                    let mut data_tx = data_channels_cloned.get(&bt.data_index()).unwrap();
                    match data_tx.tx.send(Data::BookTicker(*bt)) {
                        Ok(size) => {
                            tracing::info!("Send BookTicker, size: {:?}", size);
                        }
                        Err(e) => {
                            tracing::error!("Send BookTicker failed,{:?}", e);
                        }
                    }
                }
                _ => {}
            }
            Ok(())
        });

        Self {
            account,
            wss_streams: streams.into_iter().collect::<Vec<_>>(),
            wss: Some(wss),
            state: State {
                instances,
                optimal_price: Default::default(),
                profit: Default::default(),
                principal: Arc::new(RwLock::new(config.principal)),
                orders: Default::default(),
                archived_orders: Default::default(),
            },
            data_channels,
            decision: Default::default(),
            principal: config.principal,
            user_stream,
            order_channel: order_channel,
        }
    }

    pub async fn run(&mut self) {
        // self.run_best_price();
        // start wss
        self.run_wss().await;
        // Run instance
        self.run_instances().await;
        // Handle order
        self.run_order_monitor();
        // Handle trade signal
        self.run_trade_handle().await;
    }

    // 启动各个实例
    async fn run_instances(&mut self) {
        let instances = &mut self.state.instances;
        for instance in instances.iter_mut() {
            instance.run(&self.data_channels).await;
        }
    }

    // 启动wss数据流
    async fn run_wss(&mut self) {
        let keep_running = AtomicBool::new(true);
        // 监听数据流
        let mut streams = self.wss_streams.clone();

        match self.user_stream.start().await {
            Ok(resp) => {
                streams.push(resp.listen_key);
                tracing::info!("Join user stream");
            }
            Err(e) => {
                tracing::error!("Join user stream failed, {:?}", e);
                panic!("{:?}", e);
            }
        }

        tracing::info!("Wss subscribed, {:?}", streams);

        tokio::spawn({
            let mut wss = self.wss.take().unwrap();

            async move {
                match wss.connect_multiple(streams).await {
                    Ok(()) => {
                        tracing::info!("Wss connected");
                    }
                    Err(e) => {
                        tracing::error!("Wss connect failed, {:?}", e);
                        panic!("{:?}", e);
                    }
                }
                if let Err(e) = wss.event_loop(&keep_running).await {
                    tracing::error!("Wss stopped, {:?}", e);
                }
            }
        });
    }

    // 交易处理
    async fn run_trade_handle(&mut self) {
        let mut decision_tx = unsafe { self.decision.rx.take().unwrap_unchecked() };
        tracing::info!("Trade handle started");
        loop {
            match decision_tx.recv().await {
                Some((symbol, signal)) => {}
                None => {}
            };
        }
    }

    // 订单监控
    fn run_order_monitor(&mut self) {
        // Update order
        let mut order_rx = self.order_channel.rx.take().unwrap();
        let orders = self.state.orders.clone();
        tokio::spawn({
            let orders = orders;
            async move {
                loop {
                    if let Some(order) = order_rx.recv().await {
                        tracing::info!("Handle OrderUpdate");

                        if let Some(mut o) =
                            orders.write().await.get_mut(&order.order_id.to_string())
                        {
                            // compare time
                            if order.event_time > o.update_ts {
                                match order.current_order_status {
                                    binance::rest_model::OrderStatus::New => {
                                        o.status = OrderStatus::Accepted;
                                    }
                                    binance::rest_model::OrderStatus::Canceled => {
                                        o.status = OrderStatus::Canceled;
                                    }
                                    binance::rest_model::OrderStatus::Trade => {
                                        o.status = OrderStatus::Success;
                                    }
                                    _ => {
                                        tracing::info!("No need to handle order: {:?}", order);
                                    }
                                }
                            }
                        } else {
                            tracing::info!("The order does not belong to the engine, {:?}", order);
                        }
                    }
                }
            }
        });

        let orders = self.state.orders.clone();
        tokio::spawn({
            let mut tick = time::interval(Duration::from_secs(1));

            async move {
                loop {
                    tick.tick().await;
                    let mut orders = orders.write().await;
                    // for order in orders.keys() {
                    //     orders.remove(order);
                    // }
                }
            }
        });
    }
}
