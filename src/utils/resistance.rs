use super::teams::OpponantTeam;
use crate::{
    data::{ActivePlayer, AllPlayer},
};
use serde_json::Value;

pub struct OpponantResistances {
    pub armor: Vec<f64>,
    pub magic_resist: Vec<f64>,
}

impl<'a> OpponantResistances {
    pub fn new(
        active_player: &'a ActivePlayer,
        all_players: &'a Vec<AllPlayer>,
        ddragon_champions: &'a Value,
    ) -> Self {
        OpponantResistances {
            armor: Armor::get_scaled_ar(Armor::new(active_player, all_players, ddragon_champions)),
            magic_resist: MagicResist::get_scaled_mr(MagicResist::new(
                active_player,
                all_players,
                ddragon_champions,
            )),
        }
    }
}

struct Armor<'a> {
    opponant_team: OpponantTeam,
    ddragon_champions: &'a Value,
}

impl<'a> Armor<'a> {
    fn new(
        active_player: &'a ActivePlayer,
        all_players: &'a Vec<AllPlayer>,
        ddragon_champions: &'a Value,
    ) -> Self {
        Armor {
            opponant_team: OpponantTeam::new(active_player, all_players),
            ddragon_champions,
        }
    }
    fn get_scaled_ar(ar: Armor) -> Vec<f64> {
        // Set a Vec<f64> for opponant MR values
        let mut sar = Vec::new();
        for i in 0..ar.opponant_team.opponants.len() {
            let champion_name = &correct_name(&ar.opponant_team.opponants[i].0 as &str);
            let base_mr = ar.ddragon_champions["data"][champion_name]["stats"]["spellblock"]
                .as_f64()
                .unwrap();
            let mr_per_level = ar.ddragon_champions["data"][champion_name]["stats"]
                ["spellblockperlevel"]
                .as_f64()
                .unwrap();
            let level = ar.opponant_team.opponants[i].1 as f64;
            let scaled_ar = base_mr + (mr_per_level * (level - 1.0));
            sar.push(scaled_ar)
        }
        sar
    }
}

pub struct MagicResist<'a> {
    pub active_player: &'a ActivePlayer,
    pub all_players: &'a Vec<AllPlayer>,
    pub opponant_team: OpponantTeam,
    pub ddragon_champions: &'a Value,
}

impl<'a> MagicResist<'a> {
    pub fn new(
        active_player: &'a ActivePlayer,
        all_players: &'a Vec<AllPlayer>,
        ddragon_champions: &'a Value,
    ) -> Self {
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
            let champion_name = &correct_name(&mr.opponant_team.opponants[i].0 as &str);
            let base_mr = mr.ddragon_champions["data"][champion_name]["stats"]["spellblock"]
                .as_f64()
                .unwrap();
            let mr_per_level = mr.ddragon_champions["data"][champion_name]["stats"]
                ["spellblockperlevel"]
                .as_f64()
                .unwrap();
            let level = mr.opponant_team.opponants[i].1 as f64;
            let scaled_mr = base_mr + (mr_per_level * (level - 1.0));
            smr.push(scaled_mr)
        }
        smr
    }
}

fn correct_name(name: &str) -> String {
    match name {
        "ChoGath" => "Chogath".to_string(),
        "KhaZix" => "Khazix".to_string(),
        "KaiSa" => "Kaisa".to_string(),
        _ => name.to_string(),
    }
}
