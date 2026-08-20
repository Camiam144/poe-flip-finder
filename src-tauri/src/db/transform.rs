//! This module holds some transformation logic to take the data from the raw json
//! from the GGG API and do some stuff like attach human readable names (ggg calls
//! these "arbitrary" names but it's things like "Exalted Orb" instead of things
//! like "Metadata/Currency/CurrencyAddModToRare")
use crate::{
    db::models::{DbRow, ParsedDbRow},
    ggg_api::models::{GGGLeague, RawCxApiResponse, RawMarket},
    models::logic_models::TradingCurrencyType,
};
use anyhow::{anyhow, Result};
use std::{collections::HashMap, str::FromStr};

fn parse_raw_payload(payload: &str) -> Result<RawCxApiResponse> {
    Ok(serde_json::from_str(payload)?)
}

fn parse_market(
    raw_market: &RawMarket,
    item_name_mapping: &HashMap<String, String>,
    change_id: i64,
) -> Result<ParsedDbRow> {
    let long_name_a = raw_market.market_pair.first().unwrap();
    let long_name_b = raw_market.market_pair.get(1).unwrap();
    // TODO: Fallback to long name if it's not in the json. This kinda fricks up
    // the rest of the parsing but maybe we can fix in post? Or should we error?
    // Not sure yet.
    let short_name_a = if let Some(item) = item_name_mapping.get(long_name_a) {
        item.clone()
    } else {
        long_name_a.clone()
    };
    let short_name_b = if let Some(item) = item_name_mapping.get(long_name_b) {
        item.clone()
    } else {
        long_name_b.clone()
    };
    // INFO: Safety: Infallable
    let trading_a = TradingCurrencyType::from_str(&short_name_a).unwrap();
    let trading_b = TradingCurrencyType::from_str(&short_name_b).unwrap();

    // This errors if I can't parse, probably want to fail more gracefully.
    let currency_stats_a = extract_currency_stats(raw_market, long_name_a)?;
    let currency_stats_b = extract_currency_stats(raw_market, long_name_b)?;

    let new_row = ParsedDbRow {
        id: None,
        change_id,
        league: raw_market.league.clone(), // We know this is right because we matched it
        market_id: raw_market.market_id.clone(),
        currency_a_name_ggg: long_name_a.clone(),
        currency_b_name_ggg: long_name_b.clone(),
        currency_a_name_common: short_name_a.to_string(),
        currency_b_name_common: short_name_b.to_string(),
        volume_traded_currency_a: currency_stats_a.volume,
        volume_traded_currency_b: currency_stats_b.volume,
        lowest_stock_currency_a: currency_stats_a.lowest_stock,
        lowest_stock_currency_b: currency_stats_b.lowest_stock,
        highest_stock_currency_a: currency_stats_a.highest_stock,
        highest_stock_currency_b: currency_stats_b.highest_stock,
        lowest_ratio_currency_a: currency_stats_a.lowest_ratio,
        lowest_ratio_currency_b: currency_stats_b.lowest_ratio,
        highest_ratio_currency_a: currency_stats_a.highest_ratio,
        highest_ratio_currency_b: currency_stats_b.highest_ratio,
        is_hub_curr_a: trading_a.is_hub() as i64,
        is_hub_curr_b: trading_b.is_hub() as i64,
    };
    Ok(new_row)
}

// This struct and the following function are basically to keep all of the more
// fallible things in one spot for easier testing and error handling later.
struct CurrencyStats {
    volume: i64,
    lowest_stock: i64,
    highest_stock: i64,
    lowest_ratio: i64,
    highest_ratio: i64,
}

fn extract_currency_stats(market: &RawMarket, name: &str) -> Result<CurrencyStats> {
    Ok(CurrencyStats {
        volume: *market
            .volume_traded
            .get(name)
            .ok_or_else(|| anyhow!("Failed to get volume for {name}"))? as i64,
        lowest_stock: *market
            .lowest_stock
            .get(name)
            .ok_or_else(|| anyhow!("Failed to get lowest_stock for {name}"))?
            as i64,
        highest_stock: *market
            .highest_stock
            .get(name)
            .ok_or_else(|| anyhow!("Failed to get highest_stock for {name}"))?
            as i64,
        lowest_ratio: *market
            .lowest_ratio
            .get(name)
            .ok_or_else(|| anyhow!("Failed to get lowest_ratio for {name}"))?
            as i64,
        highest_ratio: *market
            .highest_ratio
            .get(name)
            .ok_or_else(|| anyhow!("Failed to get highest_ratio for {name}"))?
            as i64,
    })
}

pub fn clean_raw_response(
    raw_db_row: &DbRow,
    item_name_mapping: &HashMap<String, String>,
    leagues: &[GGGLeague],
) -> Result<Vec<ParsedDbRow>> {
    let raw_response = parse_raw_payload(&raw_db_row.payload)?;

    raw_response
        .markets
        .iter()
        .filter(|m| leagues.iter().any(|l| l.id == m.league))
        .map(|m| parse_market(m, item_name_mapping, raw_db_row.change_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_market(league_id: &str, pair: [&str; 2]) -> RawMarket {
        RawMarket {
            league: league_id.into(),
            market_id: format!("{}|{}", pair[0], pair[1]),
            market_pair: vec![pair[0].into(), pair[1].into()],

            volume_traded: HashMap::from([(pair[0].into(), 100), (pair[1].into(), 50)]),
            highest_ratio: HashMap::from([(pair[0].into(), 100), (pair[1].into(), 50)]),
            highest_stock: HashMap::from([(pair[0].into(), 100), (pair[1].into(), 50)]),
            lowest_ratio: HashMap::from([(pair[0].into(), 100), (pair[1].into(), 50)]),
            lowest_stock: HashMap::from([(pair[0].into(), 100), (pair[1].into(), 50)]),
        }
    }

    fn sample_item_map() -> HashMap<String, String> {
        HashMap::from([
            (
                "Metadata/Currency/CurrencyAddModToRare".into(),
                "Exalted Orb".into(),
            ),
            (
                "Metadata/Currency/CurrencyRerollRare".into(),
                "Chaos Orb".into(),
            ),
        ])
    }

    #[test]
    fn map_long_names() {
        let market = sample_market(
            "Standard",
            [
                "Metadata/Currency/CurrencyAddModToRare",
                "Metadata/Currency/CurrencyRerollRare",
            ],
        );
        let row = parse_market(&market, &sample_item_map(), 1).unwrap();

        assert_eq!(row.currency_a_name_common, "Exalted Orb");
        assert_eq!(row.currency_b_name_common, "Chaos Orb");
    }
}
