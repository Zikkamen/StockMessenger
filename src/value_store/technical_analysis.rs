use std::collections::HashMap;
use std::collections::VecDeque;

use crate::value_store::stock_information::StockInformation;
use crate::value_store::fenwick_tree::FenwickTree;

pub struct DMI {
    vec_max: VecDeque<f64>,
    vec_min: VecDeque<f64>,
    prev_close: f64,
}

impl DMI {
    pub fn new() -> Self { 
        DMI {
            vec_max: VecDeque::new(),
            vec_min: VecDeque::new(),
            prev_close: 0.0,
        }
    }
}

pub struct TechnicalAnalysis {
    time_period: f64,
    moving_average_map: HashMap<(String, usize), f64>,
    ex_moving_average_map: HashMap<(String, usize), f64>,
    dmi_map: HashMap<(String, usize), (DMI, DMI)>,
    bband_map: HashMap<(String, usize), FenwickTree>,
}

impl TechnicalAnalysis {
    pub fn new(time_period: f64) -> Self {
        TechnicalAnalysis {
            time_period: time_period,
            moving_average_map: HashMap::new(),
            ex_moving_average_map: HashMap::new(),
            dmi_map: HashMap::new(),
            bband_map: HashMap::new(),
        }
    }

    pub fn add_trade(&mut self, key: &(String, usize), stock_information: &StockInformation) {
        if key.0 != "NVDA" || key.1 != 10 {
            return;
        }

        match self.moving_average_map.get_mut(key) {
            Some(v) => *v += stock_information.avg_price,
            None => {
                self.moving_average_map.insert(key.clone(), stock_information.avg_price);
            },
        };

        let k:f64 = 2.0 / (self.time_period + 1.0);

        match self.ex_moving_average_map.get_mut(key) {
            Some(v) => *v = stock_information.avg_price * k + (*v) * (1.0 - k),
            None => {
                self.ex_moving_average_map.insert(key.clone(), stock_information.avg_price);
            },
        };

        match self.bband_map.get_mut(key) {
            Some(v) => v.insert((stock_information.avg_price * 1_000_000.0) as i64, 1),
            None => {
                let mut fenwick_tree = FenwickTree::new();
                fenwick_tree.insert((stock_information.avg_price * 1_000_000.0) as i64, 1);

                self.bband_map.insert(key.clone(), fenwick_tree);
            }
        }

        let fenwick_tree = self.bband_map.get(key).unwrap();
        let avg_price = fenwick_tree.find_nth(60);
        let upper_sigma = fenwick_tree.find_nth(90);
        let lower_sigma = fenwick_tree.find_nth(30);

        /*
        println!("SMA {}", self.moving_average_map.get(key).unwrap() / 120.0);
        println!("EMA {}", self.ex_moving_average_map.get(key).unwrap());

        println!(
            "Upper Band {} Lower Band {}", 
            (upper_sigma - avg_price) as f64 / 1_000_000.0 + stock_information.avg_price,
            stock_information.avg_price - (avg_price - lower_sigma) as f64 / 1_000_000.0,
        );

        */
    }

    pub fn remove_trade(&mut self, key: &(String, usize), stock_information: &StockInformation) {
        match self.moving_average_map.get_mut(key) {
            Some(v) => *v -= stock_information.avg_price,
            None => (),
        };

        match self.bband_map.get_mut(key) {
            Some(v) => v.insert((stock_information.avg_price * 1_000_000.0) as i64, -1),
            None => (),
        };
    }
}