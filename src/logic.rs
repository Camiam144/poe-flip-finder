use crate::models::api_models::ExchangeRecord;
use crate::models::logic_models::{TradingCurrencyRates, TradingCurrencyType};
use std::collections::HashMap;

pub fn get_base_prices(records: &[ExchangeRecord]) -> TradingCurrencyRates {
    let mut rates = TradingCurrencyRates::default();
    for record in records {
        let currency_pair = record.trading_currency();
        match currency_pair {
            (TradingCurrencyType::Divine, TradingCurrencyType::Exalt) => {
                rates.div_to_exalt = record.currency_one_data.relative_price;
            }
            (TradingCurrencyType::Chaos, TradingCurrencyType::Exalt) => {
                rates.chaos_to_exalt = record.currency_one_data.relative_price;
            }
            (_, _) => {}
        }
    }
    rates.div_to_chaos = rates.div_to_exalt / rates.chaos_to_exalt;
    rates
}
// What do we have to do once we have the values?
// I think we're going to iterate over the filtered vector one time
// Maybe we can do it after the filter step.
// for each record we get the trading currency rate and we compare
// the absolute difference of that rate to the matching tradingcurrencyrate value.
// I think we then have another struct that's like {diff, ExchangeRecord} and
// push that into a new vec. Sort that vec by absolute difference, then we can
// pretty print the output? We need to compute the expected return at some point.

pub fn build_hub_bridge_maps(
    records: &[&ExchangeRecord],
) -> (
    HashMap<(TradingCurrencyType, String), f64>,
    HashMap<(String, TradingCurrencyType), f64>,
) {
    // Build our lookup tables here so it's faster to scan every single
    // combination instead of looping through the vec of records a bazillion times
    let mut hub_to_bridge = HashMap::new();
    let mut bridge_to_hub = HashMap::new();

    for record in records {
        if let Some((hub, hub_ex, bridge_str, bridge_ex)) = record.hub_bridge_price() {
            let hub_per_bridge_ratio = hub_ex / bridge_ex;
            hub_to_bridge.insert((hub, bridge_str.clone()), hub_per_bridge_ratio);
            bridge_to_hub.insert((bridge_str, hub), hub_per_bridge_ratio.recip());
        }
    }
    (hub_to_bridge, bridge_to_hub)
}

pub fn build_bridges(
    hub_to_bridge: &HashMap<(TradingCurrencyType, String), f64>,
    bridge_to_hub: &HashMap<(String, TradingCurrencyType), f64>,
) -> Vec<(TradingCurrencyType, String, TradingCurrencyType, f64)> {
    let mut results = Vec::new();

    let hubs = [
        TradingCurrencyType::Divine,
        TradingCurrencyType::Chaos,
        TradingCurrencyType::Exalt,
    ];

    for &first_hub in &hubs {
        for &second_hub in &hubs {
            if first_hub == second_hub {
                continue;
            }
            // Now we grind through everything

            for ((_hub_one, bridge), rate_one) in
                hub_to_bridge.iter().filter(|((h, _), _)| *h == first_hub)
            {
                if let Some(rate_two) = bridge_to_hub.get(&(bridge.clone(), second_hub)) {
                    // If we have a rate two, this means we went A -> X -> B
                    // rate one is norm(A)/norm(X) and rate two is norm(X)/norm(B)
                    // so multiplying gives us norm(A)/norm(B)
                    // when B is exalts, norm(B) should be close to 1, so this
                    // will give us the relative price for A through the bridge
                    // I *think* this is right...
                    let cost = rate_one * rate_two;
                    results.push((first_hub, bridge.clone(), second_hub, cost));
                }
            }
        }
    }
    results
}

pub fn eval_profit(
    (tc1, _, tc2, rate): &(TradingCurrencyType, String, TradingCurrencyType, f64),
    ratios: &TradingCurrencyRates,
    min_profit_frac: f64,
) -> bool {
    // Profitabilty is determined by whether or not trading through the bridge
    // has an expected profit of at least `min_profit_frac` * the hub ratio.
    // This allows for flexible thresholding, for example in mid league the
    // div to exalt ratio can easily be 1:400, while the chaos to exalt ratio
    // might be closer to 1:12, setting a constant limit wouldn't show the chaos
    // trades as potentially profitable.
    let expected_hub_rate = match (tc1, tc2) {
        (TradingCurrencyType::Divine, TradingCurrencyType::Exalt) => Some(ratios.div_to_exalt),
        (TradingCurrencyType::Chaos, TradingCurrencyType::Exalt) => Some(ratios.chaos_to_exalt),
        (TradingCurrencyType::Divine, TradingCurrencyType::Chaos) => Some(ratios.div_to_chaos),
        _ => None,
    };

    if let Some(hub_rate) = expected_hub_rate {
        (rate - hub_rate).abs() >= min_profit_frac * hub_rate
    } else {
        false
    }
}

pub fn get_top_items(
    all_items: &[(TradingCurrencyType, String, TradingCurrencyType, f64)],
    currency: &TradingCurrencyType,
    num_items: usize,
) -> Vec<(TradingCurrencyType, String, TradingCurrencyType, f64)> {
    let mut output = all_items.to_vec();
    output.sort_by(|a, b| b.3.abs().total_cmp(&a.3.abs()));

    output
        .into_iter()
        .filter(|(tc1, _, _, _)| tc1 == currency)
        .take(num_items)
        .collect()
}
