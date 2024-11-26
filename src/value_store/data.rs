#[derive(Debug, Clone)]
pub struct OHLCModel {
    pub stock_name: String,
    pub price_open: f64,
    pub price_close: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub volume: f64,
    pub trades: i64,
    pub timestamp: u128,
    pub stock_interval: u128,
}

impl OHLCModel {
    pub fn from_string(s: String) -> Self {
        let mut ohlc_model = OHLCModel {
            stock_name: String::new(),
            price_open: 0.0,
            price_close: 0.0,
            min_price: 0.0,
            max_price: 0.0,
            volume: 0.0,
            trades: 0,
            timestamp: 0,
            stock_interval: 0,
        };

        let mut tmp = String::new();
        let mut i = 0;

        for c in s.chars() {
            match c {
                ';' | '\n' => {
                    match i {
                        1 => ohlc_model.stock_name = tmp,
                        2 => ohlc_model.price_open = tmp.parse::<f64>().unwrap(),
                        3 => ohlc_model.price_close = tmp.parse::<f64>().unwrap(),
                        4 => ohlc_model.min_price = tmp.parse::<f64>().unwrap(),
                        5 => ohlc_model.max_price = tmp.parse::<f64>().unwrap(),
                        6 => ohlc_model.volume = tmp.parse::<f64>().unwrap(),
                        7 => ohlc_model.trades = tmp.parse::<i64>().unwrap(),
                        8 => ohlc_model.timestamp = tmp.parse::<u128>().unwrap(),
                        9 => ohlc_model.stock_interval = tmp.parse::<u128>().unwrap(),
                        _ => (),
                    };

                    tmp = String::new();
                    i += 1;
                },
                _ => tmp.push(c),
            };
        }

        ohlc_model
    }

    pub fn to_string(&self) -> String {
        format!("{{
            \"stock_interval\": {},
            \"name\": \"{}\",
            \"price_open\": \"{:.6}\",
            \"price_close\": {:.6},
            \"min_price\": {},
            \"max_price\": {},
            \"volume\": {},
            \"trades\": {},
            \"timestamp\": {}
        }}",
            self.stock_interval,
            self.stock_name,
            self.price_open as f64,
            self.price_close as f64,
            self.min_price as f64,
            self.max_price as f64,
            self.volume,
            self.trades,
            self.timestamp,
        )
    }
}