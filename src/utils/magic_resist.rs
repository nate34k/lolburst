use crate::{active_player, all_players, get_opponant_team};


pub fn get_scaled_mr(active_player: active_player::Root, all_players: all_players::Root) -> Vec<f64> {
    // Set a Vec<f64> for opponant MR values
    let mut mr = Vec::new();
    for i in 0..get_opponant_team(&active_player, &all_players).len() {
        let champion_name = &opponant_team[i].0;
        let base_mr = ddragon_data["data"][champion_name]["stats"]["spellblock"].as_f64().unwrap();
        let mr_per_level = ddragon_data["data"][champion_name]["stats"]["spellblockperlevel"].as_f64().unwrap();
        let level = opponant_team[i].1 as f64;
        let scaled_mr = base_mr + (mr_per_level * (level - 1.0));
        mr.push(scaled_mr)
    }
}
