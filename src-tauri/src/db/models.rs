use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct DbRow {
    pub id: i64,
    pub change_id: i64,
    pub game_version: String,
    pub payload: String,
    pub parsed_bool: i64,
}

#[derive(sqlx::FromRow, Default, Debug, Clone, PartialEq, Eq)]
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
    pub domain: Option<String>,
    #[serde(rename = "drop_level")]
    pub drop_level: Option<i64>,
    pub implicits: Option<Vec<Value>>,
    #[serde(rename = "inventory_height")]
    pub inventory_height: Option<i64>,
    #[serde(rename = "inventory_width")]
    pub inventory_width: Option<i64>,
    #[serde(rename = "inherits_from")]
    pub inherits_from: Option<String>,
    #[serde(rename = "item_class")]
    pub item_class: Option<String>,
    pub name: Option<String>,
    pub properties: Option<Properties>,
    #[serde(rename = "release_state")]
    pub release_state: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "visual_identity")]
    pub visual_identity: Option<VisualIdentity>,
    pub requirements: Option<Value>,
    #[serde(rename = "grants_buff")]
    pub grants_buff: Option<Value>,
    #[serde(rename = "skills_granted")]
    pub skills_granted: Option<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub armour: Option<Value>,
    #[serde(rename = "energy_shield")]
    pub energy_shield: Option<Value>,
    pub evasion: Option<Value>,
    pub ward: Option<Value>,
    #[serde(rename = "movement_speed")]
    pub movement_speed: Option<Value>,
    pub block: Option<Value>,
    pub description: Option<String>,
    pub directions: Option<String>,
    #[serde(rename = "stack_size")]
    pub stack_size: Option<i64>,
    #[serde(rename = "stack_size_currency_tab")]
    pub stack_size_currency_tab: Option<i64>,
    #[serde(rename = "full_stack_turns_into")]
    pub full_stack_turns_into: Option<Value>,
    #[serde(rename = "charges_max")]
    pub charges_max: Option<Value>,
    #[serde(rename = "charges_per_use")]
    pub charges_per_use: Option<Value>,
    pub duration: Option<Value>,
    #[serde(rename = "life_per_use")]
    pub life_per_use: Option<Value>,
    #[serde(rename = "mana_per_use")]
    pub mana_per_use: Option<Value>,
    #[serde(rename = "attack_time")]
    pub attack_time: Option<Value>,
    #[serde(rename = "critical_strike_chance")]
    pub critical_strike_chance: Option<Value>,
    #[serde(rename = "physical_damage_max")]
    pub physical_damage_max: Option<Value>,
    #[serde(rename = "physical_damage_min")]
    pub physical_damage_min: Option<Value>,
    pub range: Option<Value>,
    #[serde(rename = "mana_burn_ms")]
    pub mana_burn_ms: Option<Value>,
    #[serde(rename = "cooldown_ms")]
    pub cooldown_ms: Option<Value>,
    #[serde(rename = "monster_id")]
    pub monster_id: Option<Value>,
    #[serde(rename = "monster_ability_text")]
    pub monster_ability_text: Option<Value>,
    #[serde(rename = "monster_category")]
    pub monster_category: Option<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualIdentity {
    #[serde(rename = "dds_file")]
    pub dds_file: Option<String>,
    pub id: Option<String>,
}
