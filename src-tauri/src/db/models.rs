use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
pub struct DbRow {
    pub id: i64,
    pub change_id: i64,
    pub game_version: String,
    pub payload: String,
}

pub struct ParsedDbRow {
    pub id: Option<i64>,
    pub change_id: i64,
    pub league: String,
    pub market_id: String,
    pub currency_a_name_ggg: String,
    pub currency_b_name_ggg: String,
    pub currency_a_name_common: String,
    pub currency_b_name_common: String,
    pub volume_traded_currency_a: i64,
    pub volume_traded_currency_b: i64,
    pub lowest_stock_currency_a: i64,
    pub lowest_stock_currency_b: i64,
    pub highest_stock_currency_a: i64,
    pub highest_stock_currency_b: i64,
    pub lowest_ratio_currency_a: i64,
    pub lowest_ratio_currency_b: i64,
    pub highest_ratio_currency_a: i64,
    pub highest_ratio_currency_b: i64,
    pub is_hub_curr_a: i64,
    pub is_hub_curr_b: i64,
}

// Need to load these in from disk basically strictly so we can get the common
// names of these items back wtf ggg.

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GGGBaseItem {
    pub domain: String,
    #[serde(rename = "drop_level")]
    pub drop_level: i64,
    pub implicits: Vec<Value>,
    #[serde(rename = "inventory_height")]
    pub inventory_height: i64,
    #[serde(rename = "inventory_width")]
    pub inventory_width: i64,
    #[serde(rename = "inherits_from")]
    pub inherits_from: String,
    #[serde(rename = "item_class")]
    pub item_class: String,
    pub name: String,
    pub properties: Properties,
    #[serde(rename = "release_state")]
    pub release_state: String,
    pub tags: Vec<String>,
    #[serde(rename = "visual_identity")]
    pub visual_identity: VisualIdentity,
    pub requirements: Value,
    #[serde(rename = "grants_buff")]
    pub grants_buff: Value,
    #[serde(rename = "skills_granted")]
    pub skills_granted: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub armour: Value,
    #[serde(rename = "energy_shield")]
    pub energy_shield: Value,
    pub evasion: Value,
    pub ward: Value,
    #[serde(rename = "movement_speed")]
    pub movement_speed: Value,
    pub block: Value,
    pub description: String,
    pub directions: String,
    #[serde(rename = "stack_size")]
    pub stack_size: i64,
    #[serde(rename = "stack_size_currency_tab")]
    pub stack_size_currency_tab: i64,
    #[serde(rename = "full_stack_turns_into")]
    pub full_stack_turns_into: Value,
    #[serde(rename = "charges_max")]
    pub charges_max: Value,
    #[serde(rename = "charges_per_use")]
    pub charges_per_use: Value,
    pub duration: Value,
    #[serde(rename = "life_per_use")]
    pub life_per_use: Value,
    #[serde(rename = "mana_per_use")]
    pub mana_per_use: Value,
    #[serde(rename = "attack_time")]
    pub attack_time: Value,
    #[serde(rename = "critical_strike_chance")]
    pub critical_strike_chance: Value,
    #[serde(rename = "physical_damage_max")]
    pub physical_damage_max: Value,
    #[serde(rename = "physical_damage_min")]
    pub physical_damage_min: Value,
    pub range: Value,
    #[serde(rename = "mana_burn_ms")]
    pub mana_burn_ms: Value,
    #[serde(rename = "cooldown_ms")]
    pub cooldown_ms: Value,
    #[serde(rename = "monster_id")]
    pub monster_id: Value,
    #[serde(rename = "monster_ability_text")]
    pub monster_ability_text: Value,
    #[serde(rename = "monster_category")]
    pub monster_category: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualIdentity {
    #[serde(rename = "dds_file")]
    pub dds_file: String,
    pub id: String,
}
