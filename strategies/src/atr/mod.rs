use std::ops::Index;

use binance::api::*;
use futures::Future;
use ndarray::{azip, s, Array1};
use rayon::prelude::*;

use crate::{Data, KlineInterval, Signal, Strategy};

pub mod backtest;

pub struct AverageTrueRange {
    period: u64,
    hlc_prices: Vec<(f64, f64, f64)>,

    pub(crate) interval: KlineInterval,
}

impl AverageTrueRange {
    pub fn new(period: u64, interval: KlineInterval) -> Self {
        Self {
            period,
            hlc_prices: vec![],
            interval,
        }
    }
    pub fn new_with_init_data() -> Self {
        Self {
            period: 14,
            hlc_prices: vec![],
            interval: KlineInterval::Day1,
        }
    }
}

impl Strategy for AverageTrueRange {
    fn signal(&mut self, data: Data) -> Signal {
        let Data::Kline(data) = data else{
            return Signal::Nothing;
        };
        // perf: 一次性合并成三元组 避免不必要的zip
        self.hlc_prices
            .push((data.kline.high, data.kline.low, data.kline.close));
        if self.hlc_prices.len() >= (self.period + 1) as usize {
            let atr = calculate_atr(&self.hlc_prices, self.period);

            // 只保存最近2*period的数据
            if self.hlc_prices.len() > (self.period * 2) as usize {
                self.hlc_prices = self
                    .hlc_prices
                    .split_off(self.hlc_prices.len() - (self.period * 2) as usize);
            }

            // 策略
            // if
        }

        Signal::Nothing
    }
}

pub fn calculate_atr(hlc_prices: &[(f64, f64, f64)], atr_period: u64) -> f64 {
    calculate_atr_by_fold(hlc_prices, atr_period)
}

pub fn calculate_atr_by_for(hlc_prices: &[(f64, f64, f64)], atr_period: u64) -> f64 {
    let mut tr_sum = 0.0;
    let mut prev_close = hlc_prices[0].2;

    for &(high, low, close) in &hlc_prices[1..] {
        let tr = (high - low).max(high - prev_close).max(prev_close - low);
        tr_sum += tr;
        prev_close = close;
    }

    let atr = tr_sum / (hlc_prices.len() - 1) as f64;

    atr
}

pub fn calculate_atr_by_fold(hlc_prices: &[(f64, f64, f64)], atr_period: u64) -> f64 {
    // perf 不使用库，避免拷贝
    let tr_sum = hlc_prices
        .windows(2)
        .map(|pair| {
            let dist1 = pair[1].0 - pair[1].1;
            let dist2 = (pair[1].0 - pair[0].2).abs();
            let dist3 = (pair[1].1 - pair[0].2).abs();
            dist1.max(dist2).max(dist3)
        })
        .fold(0., |sum, tr| sum + tr);

    tr_sum / (hlc_prices.len() - 1) as f64
}

pub fn calculate_atr_by_ndarray(hlc_prices: &[(f64, f64, f64)], atr_period: usize) -> f64 {
    let atr_period = hlc_prices.len() - 1;

    let (high, (low, close)): (Vec<_>, (Vec<_>, Vec<_>)) = hlc_prices
        .iter()
        .cloned()
        .map(|(h, l, c)| (h, (l, c)))
        .unzip();
    let high = Array1::from(high);
    let low = Array1::from(low);
    let close = Array1::from(close);
    let mut true_ranges = Array1::zeros(high.len() - 1);
    azip!((high in high.slice(s![1..]), low in low.slice(s![1..]), pc in close.slice(s![..-1]), ln in low.slice(s![1..]), mut tr in &mut true_ranges) {
        *tr = (high - low).max(high - pc).max(pc - ln);
    });

    let mut atr_sum = 0.0;
    let mut previous_tr = 0.0;
    for val in true_ranges.exact_chunks(atr_period) {
        atr_sum += val.sum();
        if previous_tr > 0.0 {
            atr_sum -= previous_tr;
        }
        previous_tr = *val.index(0);
    }

    let atr = atr_sum / atr_period as f64;
    atr
}

mod tests {
    use std::fmt::Display;

    use ta::{indicators::AverageTrueRange, Close, High, Low, Next};

    use crate::atr::{calculate_atr_by_fold, calculate_atr_by_for, calculate_atr_by_ndarray};

    #[test]
    fn test_cal() {
        struct Data {
            high: f64,
            low: f64,
            close: f64,
        }
        impl High for Data {
            fn high(&self) -> f64 {
                self.high
            }
        }
        impl Low for Data {
            fn low(&self) -> f64 {
                self.low
            }
        }
        impl Close for Data {
            fn close(&self) -> f64 {
                self.close
            }
        }
        let high = vec![47.0, 48.0, 49.0, 50.0, 51.0, 52.0];
        let low = vec![45.0, 46.0, 47.0, 48.0, 49.0, 50.0];
        let close = vec![46.0, 47.0, 48.0, 49.0, 50.0, 51.0];
        let prices = high
            .into_iter()
            .zip(low.into_iter())
            .zip(close.into_iter())
            .map(|e| (e.0 .0, e.0 .1, e.1))
            .collect::<Vec<_>>();

        let mut atr = AverageTrueRange::new(2).unwrap();
        let mut res = 0.0;
        for (h, l, c) in prices.iter() {
            res = atr.next(&Data {
                high: h.clone(),
                low: l.clone(),
                close: c.clone(),
            });
        }

        let atr = calculate_atr_by_fold(prices.as_slice(), 2);
        let atr1 = calculate_atr_by_for(prices.as_slice(), 2);
        let atr2 = calculate_atr_by_ndarray(prices.as_slice(), 2);
        assert_eq!(atr, res);
        assert_eq!(atr1, res);
        assert_eq!(atr2, res);
    }
}
