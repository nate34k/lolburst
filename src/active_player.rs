use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub abilities: Abilities,
    pub champion_stats: ChampionStats,
    pub current_gold: f64,
    pub full_runes: FullRunes,
    pub level: i64,
    pub summoner_name: String,
    pub team_relative_colors: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub passive: Passive,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ability {
    pub ability_level: i64,
    pub display_name: String,
    pub id: String,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Passive {
    pub display_name: String,
    pub id: String,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub heal_shield_power: f64,
    pub health_regen_rate: f64,
    pub life_steal: f64,
    pub magic_lethality: f64,
    pub magic_penetration_flat: f64,
    pub magic_penetration_percent: f64,
    pub magic_resist: f64,
    pub max_health: f64,
    pub move_speed: f64,
    pub omnivamp: f64,
    pub physical_lethality: f64,
    pub physical_vamp: f64,
    pub resource_max: f64,
    pub resource_regen_rate: f64,
    pub resource_type: String,
    pub resource_value: f64,
    pub spell_vamp: f64,
    pub tenacity: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullRunes {
    pub general_runes: Vec<RuneInfo>,
    pub keystone: RuneInfo,
    pub primary_rune_tree: RuneInfo,
    pub secondary_rune_tree: RuneInfo,
    pub stat_runes: Vec<StatRune>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuneInfo {
    pub display_name: String,
    pub id: i64,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRune {
    pub id: i64,
    pub raw_description: String,
}

#[derive(Default)]
pub struct AbilityRanks {
    pub q_rank: i64,
    pub w_rank: i64,
    pub e_rank: i64,
    pub r_rank: i64,
}

impl AbilityRanks {
    pub fn new(q_rank: i64, w_rank: i64, e_rank: i64, r_rank: i64) -> Self {
        AbilityRanks {
            q_rank,
            w_rank,
            e_rank,
            r_rank,
        }
    }
}
