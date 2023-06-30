use async_trait::async_trait;
use binance::{
    account::{Account, OrderCancellation, OrderRequest},
    rest_model::{OrderSide, OrderType},
};

pub(crate) struct ExpectBuy {
    pub(crate) symbol: String,
    pub(crate) price: f64,
    pub(crate) quantity: f64,
}

impl ExpectBuy {
    async fn create_limit_buy(&self, account: &Account) {
        let order = OrderRequest {
            symbol: self.symbol.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: None,
            quantity: None,
            quote_order_qty: None,
            price: Some(self.price),
            new_client_order_id: None,
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            recv_window: None,
        };
        let _resp = account.place_order(order).await;
    }
}

pub(crate) struct ExpectSell {
    pub(crate) symbol: String,
    pub(crate) price: f64,
    pub(crate) quantity: f64,
}

#[async_trait]
pub trait Handle {
    async fn handle(&self, account: &Account);
}

#[async_trait]
impl Handle for ExpectBuy {
    async fn handle(&self, account: &Account) {
        self.create_limit_buy(account).await;
    }
}

#[async_trait]
impl Handle for ExpectSell {
    async fn handle(&self, account: &Account) {
        let order = OrderRequest {
            symbol: self.symbol.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: None,
            quantity: Some(self.quantity),
            quote_order_qty: None,
            price: Some(self.price),
            new_client_order_id: None,
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            recv_window: None,
        };
        let _resp = account.place_order(order).await;
    }
}

pub(crate) struct RevokeOrder {
    pub(crate) symbol: String,
    pub(crate) order_id: String,
}

impl RevokeOrder {
    async fn revoke_order(&self, account: &Account) {
        let resp = account
            .cancel_order(OrderCancellation {
                symbol: self.symbol.clone(),
                order_id: None,
                orig_client_order_id: None,
                new_client_order_id: None,
                recv_window: None,
            })
            .await;
        match resp {
            Ok(resp) => {}
            Err(e) => {}
        }
    }
}
