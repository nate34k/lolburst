use slice_deque::SliceDeque;

use crate::{
    app::{Data, Stats},
    config::Config,
};

pub struct VS {
    pub vs_total: f64,
    pub vs_per_min: f64,
    pub vs_per_min_vecdeque: SliceDeque<(f64, f64)>,
    pub x_axis_bounds: [f64; 2],
    pub y_axis_bounds: [f64; 2],
    pub x_axis_labels: [String; 3],
    pub y_axis_labels: [String; 5],
}

impl VS {
    pub fn new() -> Self {
        VS {
            vs_total: 0.0,
            vs_per_min: 0.0,
            vs_per_min_vecdeque: SliceDeque::new(),
            x_axis_bounds: [0.0, 0.0],
            y_axis_bounds: [0.0, 2.0],
            x_axis_labels: ["-5:00".to_string(), "-2:30".to_string(), "0:00".to_string()],
            y_axis_labels: [
                0.0.to_string(),
                0.5.to_string(),
                1.0.to_string(),
                1.5.to_string(),
                2.0.to_string(),
            ],
        }
    }

    fn update_vs_total(&mut self, current_vs: f64) {
        self.vs_total = current_vs;
    }

    fn update_vs_per_min(&mut self, game_time: f64) {
        if game_time < 1.0 {
            self.vs_per_min = (self.vs_total as f64).floor() / (game_time / 60.0).ceil();
        } else {
            self.vs_per_min = (self.vs_total as f64).floor() / (game_time / 60.0);
        }
    }

    fn update_axis(&mut self) {
        let (x_front, _y_front) = self.vs_per_min_vecdeque.front().unwrap();
        let (x_back, _y_back) = self.vs_per_min_vecdeque.back().unwrap();
        self.x_axis_bounds = [*x_front, *x_back];
    }

    pub fn reset_datasets(&mut self, config: &Config, data: &Data) {
        self.vs_per_min_vecdeque = self.reset_vecdeque_dataset(config, data);
    }

    fn update_datasets(&mut self, game_time: f64) {
        self.vs_per_min_vecdeque.pop_front();
        self.vs_per_min_vecdeque
            .push_back((game_time, self.vs_per_min));
    }

    pub fn on_tick(&mut self, game_time: f64, current_vs: f64) {
        self.update_vs_total(current_vs);
        self.update_vs_per_min(game_time);
        self.update_datasets(game_time);
        self.update_axis();
    }
}

impl Stats for VS {
    fn string_from_per_min(&self) -> String {
        format!("{:.1}", self.vs_per_min)
    }
}
