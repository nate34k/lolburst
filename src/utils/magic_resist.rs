use serde_json::Value;
use crate::{active_player, all_players};
use super::teams::OpponantTeam;

pub struct MagicResist<'a> {
    pub active_player: &'a active_player::Root,
    pub all_players: &'a all_players::Root,
    pub opponant_team: OpponantTeam,
    pub ddragon_champions: &'a Value,
}

impl<'a> MagicResist<'a> {
    pub fn new(active_player: &'a active_player::Root, all_players: &'a all_players::Root, ddragon_champions: &'a Value) -> Self {
        MagicResist {
            active_player,
            all_players,
            opponant_team: OpponantTeam::new(active_player, all_players),
            ddragon_champions,
        }
    }
    pub fn get_scaled_mr(mr: MagicResist) -> Vec<f64> {
        // Set a Vec<f64> for opponant MR values
        let mut smr = Vec::new();
        for i in 0..mr.opponant_team.opponants.len() {
            let champion_name = &mr.opponant_team.opponants[i].0;
            let base_mr = mr.ddragon_champions["data"][champion_name]["stats"]["spellblock"].as_f64().unwrap();
            let mr_per_level = mr.ddragon_champions["data"][champion_name]["stats"]["spellblockperlevel"].as_f64().unwrap();
            let level = mr.opponant_team.opponants[i].1 as f64;
            let scaled_mr = base_mr + (mr_per_level * (level - 1.0));
            smr.push(scaled_mr)
        }
        smr
    }
}

