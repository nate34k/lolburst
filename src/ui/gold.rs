use std::{cmp::Ordering, time};

use slice_deque::SliceDeque;

use crate::{
    app::{Stats},
    config::Config,
};

pub struct Gold {
    pub gold_last_tick: f64,
    pub gold_total: f64,
    pub gold_per_min: f64,
    pub gold_per_min_vecdeque: SliceDeque<(f64, f64)>,
    pub x_axis_bounds: [f64; 2],
    pub y_axis_bounds: [f64; 2],
    pub x_axis_labels: [String; 3],
    pub y_axis_labels: [String; 5],
}

impl Gold {
    pub fn new() -> Self {
        Gold {
            gold_last_tick: 500.0,
            gold_total: 0.0,
            gold_per_min: 0.0,
            gold_per_min_vecdeque: SliceDeque::new(),
            x_axis_bounds: [0.0, 0.0],
            y_axis_bounds: [0.0, 600.0],
            x_axis_labels: ["-5:00".to_string(), "-2:30".to_string(), "0:00".to_string()],
            y_axis_labels: [
                0.0.to_string(),
                150.0.to_string(),
                300.0.to_string(),
                450.0.to_string(),
                600.0.to_string(),
            ],
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

    pub fn reset_dataset(&mut self, config: &Config, game_time: f64) {
        self.gold_per_min_vecdeque = self.reset_vecdeque_dataset(config, game_time);
    }

    fn update_datasets(&mut self, game_time: f64) {
        self.gold_per_min_vecdeque.pop_front();
        self.gold_per_min_vecdeque
            .push_back((game_time.round(), self.gold_per_min));
    }

    pub fn on_tick(&mut self, game_time: f64, current_gold: f64) {
        let time = time::Instant::now();
        self.update_gold_total(current_gold);
        self.update_gold_last_tick(current_gold);
        self.update_gold_per_min(game_time);
        self.update_datasets(game_time);
        self.update_axis();
        info!("Gold on_tick took: {:?}", time.elapsed());
    }
}

impl Stats for Gold {
    fn string_from_per_min(&self) -> String {
        format!("{:.1}", self.gold_per_min)
    }
}
