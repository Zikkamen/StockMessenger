
use serde_json::json;

pub struct AnalysisInfo {
    pub stocks: usize,
    pub trades: i64,
    pub volume: f64,
}

impl AnalysisInfo {
    pub fn new() -> Self {
        AnalysisInfo {
            stocks: 0,
            trades: 0,
            volume: 0.0,
        }
    }

    pub fn set_stock_number(&mut self, n: usize) {
        self.stocks = n;
    }

    pub fn reset(&mut self) -> String {
        let stock_info = json!({
            "stock_n": self.stocks,
            "trade_n": self.trades,
            "volume_n": self.volume,
        });

        self.trades = 0;
        self.volume = 0.0;

        stock_info.to_string()
    }
}