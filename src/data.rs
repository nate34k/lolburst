// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::[object Object];
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: [object Object] = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveGame {
    pub active_player: Option<ActivePlayer>,
    pub all_players: Option<Vec<AllPlayer>>,
    pub events: Option<Events>,
    pub game_data: Option<GameData>,
    pub http_status: Option<i64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePlayer {
    pub abilities: Abilities,
    pub champion_stats: ChampionStats,
    pub current_gold: f64,
    pub full_runes: FullRunes,
    pub level: i64,
    pub summoner_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Abilities {
    #[serde(rename = "Q")]
    pub q: Ability,
    #[serde(rename = "W")]
    pub w: Ability,
    #[serde(rename = "E")]
    pub e: Ability,
    #[serde(rename = "R")]
    pub r: Ability,
    #[serde(rename = "Passive")]
    pub passive: Ability,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ability {
    pub ability_level: Option<i64>,
    pub display_name: String,
    pub id: Option<String>,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionStats {
    pub ability_haste: f64,
    pub ability_power: f64,
    pub armor: f64,
    pub armor_penetration_flat: f64,
    pub armor_penetration_percent: f64,
    pub attack_damage: f64,
    pub attack_range: f64,
    pub attack_speed: f64,
    pub bonus_armor_penetration_percent: f64,
    pub bonus_magic_penetration_percent: f64,
    pub crit_chance: f64,
    pub crit_damage: f64,
    pub current_health: f64,
    pub health_regen_rate: f64,
    pub life_steal: f64,
    pub magic_lethality: f64,
    pub magic_penetration_flat: f64,
    pub magic_penetration_percent: f64,
    pub magic_resist: f64,
    pub max_health: f64,
    pub move_speed: f64,
    pub physical_lethality: f64,
    pub resource_max: f64,
    pub resource_regen_rate: f64,
    pub resource_type: String,
    pub resource_value: f64,
    pub spell_vamp: f64,
    pub tenacity: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullRunes {
    pub general_runes: Vec<Keystone>,
    pub keystone: Keystone,
    pub primary_rune_tree: Keystone,
    pub secondary_rune_tree: Keystone,
    pub stat_runes: Vec<StatRune>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keystone {
    pub display_name: String,
    pub id: i64,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRune {
    pub id: i64,
    pub raw_description: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllPlayer {
    pub champion_name: String,
    pub is_bot: bool,
    pub is_dead: bool,
    pub items: Vec<Option<serde_json::Value>>,
    pub level: i64,
    pub position: String,
    pub raw_champion_name: String,
    pub respawn_timer: f64,
    pub runes: Runes,
    pub scores: Scores,
    #[serde(rename = "skinID")]
    pub skin_id: i64,
    pub summoner_name: String,
    pub summoner_spells: SummonerSpells,
    pub team: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runes {
    pub keystone: Keystone,
    pub primary_rune_tree: Keystone,
    pub secondary_rune_tree: Keystone,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scores {
    pub assists: i64,
    pub creep_score: i64,
    pub deaths: i64,
    pub kills: i64,
    pub ward_score: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpells {
    pub summoner_spell_one: Ability,
    pub summoner_spell_two: Ability,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Events {
    #[serde(rename = "Events")]
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Event {
    #[serde(rename = "EventID")]
    pub event_id: i64,
    pub event_name: Option<String>,
    pub event_time: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game_mode: String,
    pub game_time: f64,
    pub map_name: String,
    pub map_number: i64,
    pub map_terrain: String,
}
