use std::any::{Any, TypeId};

use binance::ws_model::{BookTickerEvent, Kline};

pub enum Signal {
    Buy,
    Sell,
    Nothing,
}

pub trait Strategy{
    fn signal(&mut self, data: Data) -> Signal;
}

pub trait DataId{
    fn data_id(&self) -> TypeId ;
}

pub enum Strategies {
    RelativeStrengthIndex(rsi::RelativeStrengthIndex),
    AverageTrueRange(atr::AverageTrueRange),
}

impl Strategy for Strategies  {
    fn signal(&mut self, data: Data) -> Signal {
        match self {
            Strategies::RelativeStrengthIndex(r) => r.signal(data),
            Strategies::AverageTrueRange(a) => a.signal(data),
        }
    }
}

impl DataId for Strategies {
    fn data_id(&self) -> TypeId  {
        match self{
            Strategies::RelativeStrengthIndex(_) => TypeId::of::<Kline>(),
            Strategies::AverageTrueRange(_) => TypeId::of::<Kline>(),
        }
    }
}



#[derive(Clone)]
pub enum Data {
    Kline(Kline),
    BookTicker(BookTickerEvent),
}

// 每一个交易对对应多个策略，每个策略都不相同
// 策略模式 和 优先级模式
// 策略优先级，策略生效策略： 且 或

pub mod atr;
pub mod rsi;
// pub mod grid;
// pub mod sma;
// pub mod f4p;
// pub mod bollinger_bands;
// pub mod macd;



#[cfg(test)]
mod test {
    use std::any::Any;

    use crate::Strategy;
    use crate::{atr::AverageTrueRange, rsi::RelativeStrengthIndex, Strategies};
    #[test]
    fn test_it() {
        // let val = Strategies::AverageTrueRange(AverageTrueRange::new_with_init_data());
        // println!("{:?}", val.type_id());
        // println!(
        //     "{:?}",
        //     Strategies::RelativeStrengthIndex(RelativeStrengthIndex::default()).id()
        // );
        // println!("{:?}", AverageTrueRange::new_with_init_data().type_id());
    }
}
