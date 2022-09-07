use std::collections::VecDeque;

use crate::{app::{Stats, Data}, config::Config};

pub struct CS {
    pub cs_total: i64,
    pub cs_per_min: f64,
    pub cs_per_min_vecdeque: VecDeque<(f64, f64)>,
    pub cs_per_min_dataset: Vec<(f64, f64)>,
    pub x_axis_bounds: [f64; 2],
    pub y_axis_bounds: [f64; 2],
}

impl CS {
    pub fn new() -> Self {
        CS {
            cs_total: 0,
            cs_per_min: 0.0,
            cs_per_min_vecdeque: VecDeque::new(),
            cs_per_min_dataset: Vec::new(),
            x_axis_bounds: [0.0, 0.0],
            y_axis_bounds: [0.0, 12.0],
        }
    }

    fn update_cs_total(&mut self, current_cs: i64) {
        self.cs_total = current_cs;
    }

    fn update_cs_per_min(&mut self, game_time: f64) {
        if game_time < 1.0 {
            self.cs_per_min = (self.cs_total as f64).floor() / (game_time / 60.0).ceil();
        } else {
            self.cs_per_min = (self.cs_total as f64).floor() / (game_time / 60.0);
        }
    }

    fn update_axis(&mut self) {
        let (x_front, _y_front) = self.cs_per_min_vecdeque.front().unwrap();
        let (x_back, _y_back) = self.cs_per_min_vecdeque.back().unwrap();
        self.x_axis_bounds = [*x_front, *x_back];
    }

    pub fn reset_datasets(&mut self, config: &Config, data: &Data) {
        self.cs_per_min_vecdeque = self.reset_vecdeque_dataset(config, data);
        self.cs_per_min_dataset = self.reset_vec_dataset(config);
    }

    fn update_datasets(&mut self, game_time: f64) {
        self.cs_per_min_vecdeque.pop_front();
        self.cs_per_min_vecdeque.push_back((game_time, self.cs_per_min));
        self.cs_per_min_dataset = self.cs_per_min_vecdeque.iter().map(|(x, y)| (*x, *y)).collect();
    }

    pub fn on_tick(&mut self, game_time: f64, current_cs: i64) {
        self.update_cs_total(current_cs);
        self.update_cs_per_min(game_time);
        self.update_datasets(game_time);
        self.update_axis();
    }
}

impl Stats for CS {
    fn string_from_per_min(&self) -> String {
        format!("{:.1}", self.cs_per_min)
    }
}