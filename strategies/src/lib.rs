use std::hash::Hash;

use binance::ws_model::{BookTickerEvent, Kline, KlineEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Signal {
    Buy,
    Sell,
    Nothing,
}

pub trait Strategy {
    fn signal(&mut self, data: Data) -> Signal;
}

/// 数据唯一性确定
pub trait DataId {
    fn data_id(&self) -> u64;
}

/// 数据分类
pub trait DataCategory {
    fn data_category(&self) -> Category;
}

pub enum Strategies {
    RelativeStrengthIndex(rsi::RelativeStrengthIndex),
    AverageTrueRange(atr::AverageTrueRange),
}

impl Strategy for Strategies {
    fn signal(&mut self, data: Data) -> Signal {
        match self {
            Strategies::RelativeStrengthIndex(r) => r.signal(data),
            Strategies::AverageTrueRange(a) => a.signal(data),
        }
    }
}

impl DataCategory for Strategies {
    fn data_category(&self) -> Category {
        match self {
            Strategies::RelativeStrengthIndex(rsi) => Category::Kline(rsi.interval.clone()),
            Strategies::AverageTrueRange(atr) => Category::Kline(atr.interval.clone()),
        }
    }
}

impl DataCategory for &Strategies {
    fn data_category(&self) -> Category {
        match self {
            Strategies::RelativeStrengthIndex(rsi) => Category::Kline(rsi.interval.clone()),
            Strategies::AverageTrueRange(atr) => Category::Kline(atr.interval.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Data {
    Kline(KlineEvent),
    BookTicker(BookTickerEvent),
}

impl DataId for Data {
    fn data_id(&self) -> u64 {
        match self {
            Data::Kline(k) => k.event_time,
            Data::BookTicker(b) => b.update_id,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum Category {
    BookTicker,
    Kline(KlineInterval),
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize, Eq, PartialEq)]
pub enum KlineInterval {
    #[serde(rename = "1h")]
    Hour1,
    #[serde(rename = "2h")]
    Hour2,
    #[serde(rename = "4h")]
    Hour4,
    #[serde(rename = "6h")]
    Hour6,
    #[serde(rename = "1d")]
    Day1,
}

impl KlineInterval {
    pub fn to_string(&self) -> String {
        match self {
            KlineInterval::Hour1 => "1h".to_string(),
            KlineInterval::Hour2 => "2h".to_string(),
            KlineInterval::Hour4 => "4h".to_string(),
            KlineInterval::Hour6 => "6h".to_string(),
            KlineInterval::Day1 => "1d".to_string(),
        }
    }
}

impl DataCategory for Kline {
    fn data_category(&self) -> Category {
        match self.interval.as_str() {
            "1h" => Category::Kline(KlineInterval::Hour1),
            "2h" => Category::Kline(KlineInterval::Hour2),
            "4h" => Category::Kline(KlineInterval::Hour4),
            "6h" => Category::Kline(KlineInterval::Hour6),
            _ => Category::Kline(KlineInterval::Day1),
        }
    }
}

impl DataCategory for KlineEvent {
    fn data_category(&self) -> Category {
        self.kline.data_category()
    }
}

impl DataCategory for BookTickerEvent {
    fn data_category(&self) -> Category {
        Category::BookTicker
    }
}

impl DataCategory for Data {
    fn data_category(&self) -> Category {
        match self {
            Data::Kline(k) => k.kline.data_category(),
            Data::BookTicker(b) => b.data_category(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Index<Key = String>(Key, Category)
where
    Key: PartialEq + Eq + Hash + Clone;

pub trait DataIndex<Key>
where
    Key: PartialEq + Eq + Hash + Clone,
{
    fn data_index(&self) -> Index<Key>;
}

impl<Key, DC> DataIndex<Key> for (Key, DC)
where
    DC: DataCategory,
    Key: PartialEq + Eq + Hash + Clone,
{
    fn data_index(&self) -> Index<Key> {
        Index::<Key>(self.0.clone(), self.1.data_category())
    }
}

impl<Key> DataIndex<Key> for (Key, Category)
where
    Key: PartialEq + Eq + Hash + Clone,
{
    fn data_index(&self) -> Index<Key> {
        Index::<Key>(self.0.clone(), self.1.clone())
    }
}

impl DataIndex<String> for KlineEvent {
    fn data_index(&self) -> Index<String> {
        Index(self.symbol.clone(), self.data_category())
    }
}

impl DataIndex<String> for BookTickerEvent {
    fn data_index(&self) -> Index<String> {
        Index(self.symbol.clone(), self.data_category())
    }
}

pub mod atr;
pub mod rsi;
// pub mod grid;
// pub mod sma;
// pub mod f4p;
// pub mod bollinger_bands;
// pub mod macd;

#[cfg(test)]
mod test {
    use crate::Category;

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
