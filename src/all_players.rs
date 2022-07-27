use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub all_players: Vec<AllPlayer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllPlayer {
    pub champion_name: String,
    pub is_bot: bool,
    pub is_dead: bool,
    pub items: Vec<Item>,
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
    pub raw_skin_name: Option<String>,
    pub skin_name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub can_use: bool,
    pub consumable: bool,
    pub count: i64,
    pub display_name: String,
    #[serde(rename = "itemID")]
    pub item_id: i64,
    pub price: i64,
    pub raw_description: String,
    pub raw_display_name: String,
    pub slot: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runes {
    pub keystone: Keystone,
    pub primary_rune_tree: PrimaryRuneTree,
    pub secondary_rune_tree: SecondaryRuneTree,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keystone {
    pub display_name: String,
    pub id: i64,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryRuneTree {
    pub display_name: String,
    pub id: i64,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecondaryRuneTree {
    pub display_name: String,
    pub id: i64,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scores {
    pub assists: i64,
    pub creep_score: i64,
    pub deaths: i64,
    pub kills: i64,
    pub ward_score: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpells {
    pub summoner_spell_one: SummonerSpellOne,
    pub summoner_spell_two: SummonerSpellTwo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpellOne {
    pub display_name: String,
    pub raw_description: String,
    pub raw_display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpellTwo {
    pub display_name: String,
    pub raw_description: String,
    pub raw_display_name: String,
}
