use std::collections::VecDeque;
use serde_json::json;

use crate::value_store::OHLCModel;

pub struct AnalysisInfo {
    stocks: usize,
    trades: i64,
    volume: f64,
    market_value: f64,
    data_history: VecDeque<String>,
}

impl AnalysisInfo {
    pub fn new() -> Self {
        AnalysisInfo {
            stocks: 0,
            trades: 0,
            volume: 0.0,
            market_value: 0.0,
            data_history: VecDeque::new(),
        }
    }

    pub fn add_ohlc(&mut self, ohlc_model: &OHLCModel) {
        self.trades += ohlc_model.trades;
        self.volume += ohlc_model.volume;
        self.market_value += ohlc_model.price_close * ohlc_model.volume as f64;
    }

    pub fn set_stock_number(&mut self, n: usize) {
        self.stocks = n;
    }

    pub fn reset(&mut self, timestamp: u128) -> String {
        let stock_info = json!({
            "stock_n": self.stocks,
            "trade_n": self.trades,
            "volume_n": self.volume,
            "market_v_n": self.market_value,
            "timestamp": timestamp,
        }).to_string();

        self.trades = 0;
        self.volume = 0.0;
        self.market_value = 0.0;

        self.data_history.push_back(stock_info.clone());
        
        if self.data_history.len() > 120 {
            let _ = self.data_history.pop_front();
        }

        stock_info
    }

    pub fn get_history(&self) -> Vec<String> {
        let mut data_history = Vec::<String>::new();

        for info in self.data_history.iter() {
            data_history.push(info.clone());
        }

        data_history.push("End of Update".to_owned());

        data_history
    }
}