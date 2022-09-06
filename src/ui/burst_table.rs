use serde_json::Value;

use crate::{
    active_player::AbilityRanks,
    app,
    champion::{self, CalculateDamageTrait, Champion},
    dmg,
    utils::{resistance, teams},
};

pub struct BurstTable<'a> {
    pub champion: &'a Champion<'a>,
    pub data: &'a app::Data,
    pub data_dragon_data: &'a Value,
    pub rotation: &'a str,
}

impl BurstTable<'_> {
    // Function to build a Vec<Vec<String>> for the burst table widget
    pub fn build_burst_table_items(self) -> Vec<Vec<String>> {
        // Set ability_ranks to new AbilityRanks
        // Used to calculate burst damage
        let ability_ranks = AbilityRanks::new(
            self.data.active_player_data.abilities.q.ability_level,
            self.data.active_player_data.abilities.w.ability_level,
            self.data.active_player_data.abilities.e.ability_level,
            self.data.active_player_data.abilities.r.ability_level,
        );

        // Set opponance_resistances to new OpponantResistances
        // Used to calculate burst damage
        let opponant_resistances = resistance::OpponantResistances::new(
            &self.data.active_player_data,
            &self.data.all_player_data,
            self.data_dragon_data,
        );

        // Set opponent_team to new OpponentTeam
        // Used to format the burst table
        let opponant_team =
            teams::OpponantTeam::new(&self.data.active_player_data, &self.data.all_player_data);

        // Set ret as a Vec<Vec<String>>
        // This is the data type that the table widget expects
        let mut ret: Vec<Vec<String>> = Vec::new();

        // Loop to push Vec<String> to ret:
        for i in 0..opponant_team.opponants.len() {
            // Set row to a new Vec<String>
            let mut row = Vec::new();

            // Set r to a new dmg::Resistance with the resistance values for the current champion
            // Used to calculate burst damage
            let r = dmg::Resistance::new(
                opponant_resistances.armor[i],
                opponant_resistances.magic_resist[i],
            );

            // Set burst_dmg to the calculated burst damage for the current champion
            // Used to format the burst table
            let burst_dmg = self
                .champion
                .calculate_damage(champion::CalculateDamageStruct {
                    active_player: &self.data.active_player_data,
                    ability_ranks: &ability_ranks,
                    resistance: &r,
                    rotation: self.rotation,
                });

            // Push the champion's name to row
            row.push(opponant_team.opponants[i].0.clone());

            // Push the champion's level to row
            row.push(opponant_team.opponants[i].1.to_string());

            // Push the calculated burst damage to row
            row.push(burst_dmg.floor().to_string());

            // Push row to ret
            ret.push(row);
        }

        // Return ret
        ret
    }
}
