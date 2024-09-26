pub struct StockInformation {
    pub stock_name: String,
    pub stock_interval: usize,
    pub timestamp:i64,

    pub avg_price: f64,
    pub avg_price_open: f64,
    pub min_price: f64,
    pub max_price: f64,

    pub volume_moved: i64,
    pub num_of_trades: i64,
}

impl StockInformation {
    pub fn new() -> Self {
        StockInformation { 
            stock_name: String::new(),
            stock_interval: 0,
            timestamp: 0,
            avg_price: 0.0,
            avg_price_open: 0.0,
            min_price: 0.0,
            max_price: 0.0,
            volume_moved: 0,
            num_of_trades: 0
        }
    }

    pub fn clone(&self) -> Self {
        StockInformation {
            stock_name: self.stock_name.clone(),
            stock_interval: self.stock_interval,
            timestamp: self.timestamp,
            avg_price: self.avg_price,
            avg_price_open: self.avg_price_open,
            min_price: self.min_price,
            max_price: self.max_price,
            volume_moved: self.volume_moved,
            num_of_trades: self.num_of_trades
        }
    }

    pub fn insert_data(&mut self, key: String, value: String) {
        match key.as_str() {
            "sn" => self.stock_name = value,
            "si" => self.stock_interval = value.parse::<usize>().unwrap(),
            "ap" => self.avg_price = value.parse::<f64>().unwrap(),
            "op" => self.avg_price_open = value.parse::<f64>().unwrap(),
            "mn" => self.min_price = value.parse::<f64>().unwrap(),
            "mx" => self.max_price = value.parse::<f64>().unwrap(),
            "vm" => self.volume_moved = value.parse::<i64>().unwrap(),
            "nt" => self.num_of_trades = value.parse::<i64>().unwrap(),
            "t" => self.timestamp = value.parse::<i64>().unwrap(),
            _ => (),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{{
            \"stock_interval\": {},
            \"name\": \"{}\",
            \"avg_price\": \"{:.6}\",
            \"avg_price_open\": {:.6},
            \"min_price\": {},
            \"max_price\": {},
            \"volume_moved\": {},
            \"num_of_trades\": {},
            \"timestamp\": {}
        }}",
            self.stock_interval,
            self.stock_name,
            self.avg_price as f64,
            self.avg_price_open as f64,
            self.min_price as f64,
            self.max_price as f64,
            self.volume_moved,
            self.num_of_trades,
            self.timestamp,
        )
    }
}