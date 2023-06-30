use binance::api::*;
use ndarray::{s, Array1};
use rayon::prelude::*;
use ta::Next;

use crate::{Data, KlineInterval, Signal, Strategy};

pub mod backtest;

pub struct RelativeStrengthIndex {
    close_prices: Vec<f64>,

    period: u64,
    buy_threshold: f64, // default
    sell_threshold: f64,

    pub(crate) interval: KlineInterval,
}

impl RelativeStrengthIndex {
    pub fn new(
        period: u64,
        interval: KlineInterval,
        buy_threshold: f64,
        sell_threshold: f64,
    ) -> Self {
        Self {
            close_prices: vec![],
            period,
            interval,
            buy_threshold,
            sell_threshold,
        }
    }
}

impl Default for RelativeStrengthIndex {
    fn default() -> Self {
        Self {
            close_prices: vec![],
            period: 14,
            buy_threshold: 30.,
            sell_threshold: 70.,
            interval: KlineInterval::Day1,
        }
    }
}

impl Strategy for RelativeStrengthIndex {
    fn signal(&mut self, data: Data) -> Signal {
        let Data::Kline(data) = data else{
            return Signal::Nothing;
        };
        let close_price: f64 = data.kline.close;
        self.close_prices.push(close_price);

        if self.close_prices.len() >= (self.period + 1) as usize {
            let rsi = calculate_rsi(&self.close_prices, self.period as usize);

            // 保留最近的数据
            if self.close_prices.len() > (self.period * 2) as usize {
                self.close_prices = self
                    .close_prices
                    .split_off(self.close_prices.len() - (self.period * 2) as usize);
            }

            if rsi <= self.buy_threshold {
                return Signal::Buy;
            } else if rsi >= self.sell_threshold {
                return Signal::Sell;
            }
        }

        Signal::Nothing
    }
}

pub fn calculate_rsi(close_prices: &[f64], period: usize) -> f64 {
    calculate_rsi_by_rayon(close_prices, period)
}

pub fn calculate_rsi_by_for(prices: &[f64], period: usize) -> f64 {
    let period = prices.len() - 1;
    let (prev, gain_sum, loss_sum) =
        prices
            .iter()
            .fold((&0., 0., 0.), |(prev, gain_sum, loss_sum), cur| {
                if prev == &0. {
                    (cur, gain_sum, loss_sum)
                } else {
                    let diff = cur - prev;
                    if diff >= 0. {
                        (cur, gain_sum + diff, loss_sum)
                    } else {
                        (cur, gain_sum, loss_sum + diff.abs())
                    }
                }
            });

    let mut avg_gain = gain_sum / period as f64;
    let mut avg_loss = loss_sum / period as f64;

    (avg_gain / (avg_gain + avg_loss)) * 100.
}

pub fn calculate_rsi_by_ndarray(prices: &[f64], period: usize) -> f64 {
    let period = prices.len() - 1;
    let prices = Array1::from_vec(prices.to_vec());

    let gain_loss = &prices.slice(s![..-1]) - &prices.slice(s![1..]);
    let (gain_sum, loss_sum) =
        gain_loss
            .iter()
            .fold((0.0, 0.0), |(sum_gain, sum_loss), &change| {
                if change >= 0.0 {
                    (sum_gain + change, sum_loss)
                } else {
                    (sum_gain, sum_loss + change.abs())
                }
            });

    let avg_gain = gain_sum / period as f64;
    let avg_loss = loss_sum / period as f64;

    (avg_gain / (avg_gain + avg_loss)) * 100.
}

pub fn calculate_rsi_by_rayon(prices: &[f64], period: usize) -> f64 {
    let period = prices.len() - 1;
    let (gain_sum, loss_sum): (f64, f64) = prices
        .par_windows(2)
        .map(|e| e[1] - e[0])
        .fold(
            || (0.0, 0.0),
            |(sum_gain, sum_loss), change| {
                if change >= 0.0 {
                    (sum_gain + change, sum_loss)
                } else {
                    (sum_gain, sum_loss + change.abs())
                }
            },
        )
        .reduce(
            || (0.0, 0.0),
            |(gain1, loss1), (gain2, loss2)| {
                // println!("{:?},{:?}", gain1, loss1);
                (gain1 + gain2, loss1 + loss2)
            },
        );

    let avg_gain = gain_sum / period as f64;
    let avg_loss = loss_sum / period as f64;

    (avg_gain / (avg_gain + avg_loss)) * 100.
}

pub fn calculate_rsi_by_rayon_and_ndarray(prices: &[f64], period: usize) -> f64 {
    let period = prices.len() - 1;
    let prices = Array1::from_vec(prices.to_vec());
    let changes: Array1<f64> = &prices.slice(s![1..]) - &prices.slice(s![..-1]);
    let (gain_sum, loss_sum) = changes
        .par_iter()
        .fold(
            || (0.0, 0.0),
            |(sum_gain, sum_loss), change| {
                if change >= &0.0 {
                    (sum_gain + change, sum_loss)
                } else {
                    (sum_gain, sum_loss + change.abs())
                }
            },
        )
        .reduce(
            || (0.0, 0.0),
            |(gain1, loss1), (gain2, loss2)| (gain1 + gain2, loss1 + loss2),
        );

    let avg_gain = gain_sum / period as f64;
    let avg_loss = loss_sum / period as f64;

    (avg_gain / (avg_gain + avg_loss)) * 100.
}

mod tests {
    use ndarray::array;
    use ta::{indicators::RelativeStrengthIndex, Next};

    use super::calculate_rsi_by_ndarray;
    use crate::rsi::{
        calculate_rsi_by_for, calculate_rsi_by_rayon, calculate_rsi_by_rayon_and_ndarray,
    };

    #[test]
    fn test_it() {
        let prices = array![
            46.125, 47.125, 46.4375, 46.9375, 44.9375, 44.25, 44.625, 45.75, 47.8125, 47.5625,
            47.0, 44.5625, 46.3125, 47.6875, 46.6875, 45.6875, 43.0625
        ];
        let prices = vec![
            46.125, 47.125, 46.4375, 46.9375, 44.9375, 44.25, 44.625, 45.75, 47.8125, 47.5625,
            47.0, 44.5625, 46.3125, 47.6875, 46.6875, 45.6875, 43.0625,
        ];

        let mut rsi = RelativeStrengthIndex::new(14).unwrap();
        let mut rsi_value = 0.;
        for &price in &prices[..15] {
            rsi_value = rsi.next(price);
        }
        let rsi = calculate_rsi_by_ndarray(&prices[..15], 14);
        let rsi1 = calculate_rsi_by_rayon(&prices[..15], 14);
        let rsi2 = calculate_rsi_by_rayon_and_ndarray(&prices[..15], 14);
        let rsi3 = calculate_rsi_by_for(&prices[..15], 14);

        println!("ta:{}", rsi_value);
        println!("ndarray:{}", rsi);
        println!("rayon:{}", rsi1);
        println!("rn:{}", rsi2);
        println!("for:{}", rsi3);
        assert_eq!(rsi1, rsi);
    }
}
