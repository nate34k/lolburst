#[derive(Default)]
pub struct Resistance {
    pub armor: f64,
    pub magic_resist: f64,
}

impl Resistance {
    pub fn new(armor: f64, magic_resist: f64) -> Self {
        Resistance {
            armor,
            magic_resist,
        }
    }
}

pub fn calculate_mitigation(rd: f64, resistance: f64) -> f64 {
    rd / (1.0 + (resistance / 100.0))
}

pub fn mitigate_damage_by_armor(damage: f64, armor: f64) -> f64 {
    calculate_mitigation(damage, armor)
}

pub fn mitigate_damage_by_magic_resist(damage: f64, magic_resist: f64) -> f64 {
    calculate_mitigation(damage, magic_resist)
}
