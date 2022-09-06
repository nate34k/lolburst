use std::{cmp::Ordering, collections::VecDeque};

use crate::{
    app::{Data, Stats},
    config::Config,
};

pub struct Gold {
    pub gold_last_tick: f64,
    pub gold_total: f64,
    pub gold_per_min: f64,
    pub gold_per_min_vecdeque: VecDeque<(f64, f64)>,
    pub gold_per_min_dataset: Vec<(f64, f64)>,
    pub x_axis_bounds: [f64; 2],
    pub y_axis_bounds: [f64; 2],
}

impl Gold {
    pub fn new() -> Self {
        Gold {
            gold_last_tick: 500.0,
            gold_total: 0.0,
            gold_per_min: 0.0,
            gold_per_min_vecdeque: VecDeque::new(),
            gold_per_min_dataset: Vec::new(),
            x_axis_bounds: [0.0, 0.0],
            y_axis_bounds: [0.0, 600.0],
        }
    }

    fn update_gold_total(&mut self, current_gold: f64) {
        if let Some(Ordering::Greater) = current_gold.partial_cmp(&self.gold_last_tick) {
            self.gold_total += current_gold - self.gold_last_tick;
        }
    }

    fn update_gold_per_min(&mut self, game_time: f64) {
        if game_time < 1.0 {
            self.gold_per_min = self.gold_total.floor() / (game_time / 60.0).ceil();
        } else {
            self.gold_per_min = self.gold_total.floor() / (game_time / 60.0);
        }
    }

    fn update_gold_last_tick(&mut self, current_gold: f64) {
        self.gold_last_tick = current_gold;
    }

    fn update_axis(&mut self) {
        let (x_front, _y_front) = self.gold_per_min_vecdeque.front().unwrap();
        let (x_back, _y_back) = self.gold_per_min_vecdeque.back().unwrap();
        self.x_axis_bounds = [*x_front, *x_back];
    }

    pub fn reset_datasets(&mut self, config: &Config, data: &Data) {
        self.gold_per_min_vecdeque = self.reset_vecdeque_dataset(config, data);
        self.gold_per_min_dataset = self.reset_vec_dataset(config);
    }

    pub fn on_tick(&mut self, game_time: f64, current_gold: f64) {
        self.update_gold_total(current_gold);
        self.update_gold_last_tick(current_gold);
        self.update_gold_per_min(game_time);
        self.gold_per_min_vecdeque.pop_front();
        self.gold_per_min_vecdeque
            .push_back((game_time.round(), self.gold_per_min));
        self.update_axis();
        self.gold_per_min_dataset = Vec::from(self.gold_per_min_vecdeque.clone());
    }
}

impl Stats for Gold {
    fn string_from_per_min(&self) -> String {
        format!("{:.1}", self.gold_per_min)
    }
}
