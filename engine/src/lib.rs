use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicBool, Arc},
};

use binance::{
    account::{Account, OrderRequest},
    api::Binance,
    rest_model::{OrderSide, OrderType},
    websockets::{book_ticker_stream, kline_stream, WebSockets},
    ws_model::{BookTickerEvent, CombinedStreamEvent, Kline, WebsocketEvent, WebsocketEventUntag},
};
use strategies::{Data, DataId, Signal, Strategies, Strategy};
use tokio::sync::{broadcast, mpsc, RwLock};

pub mod config;

pub struct Engine<'s> {
    account: Account,
    data_channels: HashMap<TypeId, DataStream>,
    state: State<'s>,
    wss: WebSockets<'s, CombinedStreamEvent<WebsocketEventUntag>>,
}

pub struct State<'s> {
    symbols: Vec<Instance>,
    best_price: Arc<RwLock<HashMap<&'s str, (f64, f64)>>>,
}

impl Engine<'_> {
    pub fn new(api_key: &str, secret_key: &str) -> Self {
        // 初始化账户
        let account = Account::new(Some(api_key.to_string()), Some(secret_key.to_string()));
        // 数据通道
        let mut data_channels = HashMap::new();

        let (kline_tx, _kline_rx) = broadcast::channel::<Data>(1024);
        let (book_ticker_tx, _book_ticker_rx) = broadcast::channel::<Data>(1024);

        data_channels.insert(TypeId::of::<Kline>(), kline_tx.clone());
        data_channels.insert(TypeId::of::<BookTickerEvent>(), book_ticker_tx.clone());

        // 监听数据流
        //        let kline_tx_clone = kline_tx.clone();
        //        let book_ticker_tx_clone = book_ticker_tx.clone();
        let wss = WebSockets::new(move |e: CombinedStreamEvent<WebsocketEventUntag>| {
            match e.data {
                WebsocketEventUntag::WebsocketEvent(WebsocketEvent::Kline(kline)) => {
                    kline_tx.send(Data::Kline(kline.kline));
                }
                WebsocketEventUntag::BookTicker(bt) => {
                    book_ticker_tx.send(Data::BookTicker(*bt));
                }
                _ => {}
            }
            Ok(())
        });

        Self {
            account,
            wss,
            state: State {
                symbols: Default::default(),
                best_price: Default::default(),
            },
            data_channels,
        }
    }

    // 增加交易对
    pub fn add_symbol(&mut self, _symbol: &str, _strategies: Vec<Strategies>) {}

    // 创建卖出限价单
    async fn create_sell_limit_order(&self, symbol: &str, price: f64) {
        let order = OrderRequest {
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: None,
            quantity: None,
            quote_order_qty: None,
            price: Some(price),
            new_client_order_id: None,
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            recv_window: None,
        };
        let _resp = self.account.place_order(order).await;
    }

    // 创建买入限价单
    async fn create_buy_limit_order(&self, symbol: &str, price: f64) {
        let order = OrderRequest {
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: None,
            quantity: None,
            quote_order_qty: None,
            price: Some(price),
            new_client_order_id: None,
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            recv_window: None,
        };
        let _resp = self.account.place_order(order).await;
    }

    pub async fn run(&mut self) {
        let keep_running = AtomicBool::new(true);

        // Handle trade signal
        
        // Run instance
        for instance in self.state.symbols.iter_mut() {
            instance.run().await;
        }

        let mut streams = Vec::new();
        for symbol in self.state.symbols.iter() {
            streams.push(kline_stream(&symbol.symbol, "1h"));
            streams.push(book_ticker_stream(&symbol.symbol));
        }
        self.wss.connect_multiple(streams).await.unwrap();
        if let Err(e) = self.wss.event_loop(&keep_running).await {
            println!("Error: {e}");
        }
    }
    
    pub async fn handle_trade(&self){
        
    }
}

pub struct Instance {
    symbol: String,
    strategies: Vec<Arc<RwLock<Strategies>>>,
    strategy_mode: StrategyMode,
    signal_channel: (
        mpsc::UnboundedSender<(String, Signal)>,
        mpsc::UnboundedReceiver<(String, Signal)>,
    ),
}

pub struct SignalRequest {
    strategy_name: String,
    signal: Signal,
    ts: i64,
}

impl Instance {
    pub fn new(symbol: &str, mode: StrategyMode, strategies: HashSet<Strategies>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<(String, Signal)>();
        Self {
            symbol: symbol.to_string(),
            strategies: strategies
                .into_iter()
                .map(|e| Arc::new(RwLock::new(e)))
                .collect::<Vec<_>>(),
            strategy_mode: mode,
            signal_channel: (tx, rx),
        }
    }

    pub async fn run(&mut self,engine: &Engine<'_>) {
        
        self.run_strategies(engine).await;
        // 处理交易信号
        while let Some(_signal) = self.signal_channel.1.recv().await {
            
        }
    }

    pub async fn run_strategies(&self, engine: &Engine<'_>) {
        for strategy in self.strategies.iter().map(Clone::clone) {
            let signal_tx = self.signal_channel.0.clone();
            let data_id = { strategy.read().await.data_id() };

            let mut data_rx = engine.data_channels.get(&data_id).unwrap().subscribe();
            tokio::spawn({
                async move {
                    loop {
                        match data_rx.recv().await {
                            Ok(data) => {
                                let signal = strategy.write().await.signal(data);
                                signal_tx.send(("message".to_string(), signal));
                            }
                            Err(_err) => {
                                break;
                            }
                        }
                    }
                }
            });
        }
    }
}

// 策略生效模式
#[derive(Default)]
pub enum StrategyMode {
    #[default]
    Or,
    // And,
    // Weight,
}

type DataStream = broadcast::Sender<Data>;

mod test {
    use crate::Engine;

    #[tokio::test]
    async fn test_it() {
        let mut engine = Engine::new("", "");
        engine.run().await;
    }
}
