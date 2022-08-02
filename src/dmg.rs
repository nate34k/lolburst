use crate::{
    active_player::{self, AbilityRanks},
    champions::ActiveChampion,
};

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

pub fn burst_dmg(
    active_champion: &ActiveChampion,
    active_player: &active_player::Root,
    ability_ranks: &AbilityRanks,
    resistance: Resistance,
) -> f64 {
    match active_champion {
        ActiveChampion::Orianna(orianna) => crate::orianna::Orianna::calculate_damage(
            orianna,
            active_player,
            std::env::var("ROTATION").unwrap().as_str(),
            ability_ranks,
            resistance,
        ),
        _ => 0.0,
    }
}

pub fn calculate_mitigation(rd: f64, resistance: f64) -> f64 {
    rd / (1.0 + (resistance / 100.0))
}
