pub mod models;

use models::{
    ArbitrageOpportunity, Graph, Market, TradingCurrencyRates, TradingCurrencyType, TradingEdge,
};

fn avg_exchange_rate(
    market: &Market,
    from: &TradingCurrencyType,
    to: &TradingCurrencyType,
) -> Option<f64> {
    let vol_a = market.volume_traded_currency_a as f64;
    let vol_b = market.volume_traded_currency_b as f64;

    match (&market.currency_a, &market.currency_b) {
        (a, b) if a == from && b == to => Some(vol_b / vol_a),
        (a, b) if a == to && b == from => Some(vol_a / vol_b),
        _ => None,
    }
}

/// Get the average price of each actual trading currency.
/// These volumes should be high enough that there is basically zero inefficiency.
pub fn get_base_prices(markets: &[Market]) -> TradingCurrencyRates {
    let mut rates = TradingCurrencyRates::default();

    for market in markets.iter().filter(|m| m.is_trading_rate()) {
        if let Some(rate) = avg_exchange_rate(
            market,
            &TradingCurrencyType::Divine,
            &TradingCurrencyType::Exalt,
        ) {
            rates.div_to_exalt = rate;
        } else if let Some(rate) = avg_exchange_rate(
            market,
            &TradingCurrencyType::Divine,
            &TradingCurrencyType::Chaos,
        ) {
            rates.div_to_chaos = rate;
        } else if let Some(rate) = avg_exchange_rate(
            market,
            &TradingCurrencyType::Chaos,
            &TradingCurrencyType::Exalt,
        ) {
            rates.chaos_to_exalt = rate;
        }
    }
    rates
}

/// Build a graph, this will be the core of the logic
pub fn build_and_populate_graph(markets: &[Market]) -> Graph {
    let mut graph = Graph::new();

    for market in markets {
        // If we have no volume or some weird no ratio issue, just skip it for this
        // set of markets, we don't want to trade it anyway.
        if market.volume_traded_currency_a == 0
            || market.volume_traded_currency_b == 0
            || market.get_lowest_ratio().is_none()
            || market.get_highest_ratio().is_none()
            || market.get_lowest_ratio().is_some_and(|m| m == 0.0)
            || market.get_highest_ratio().is_some_and(|m| m == 0.0)
        {
            continue;
        }
        // Safety: Already checked for none vals for ratios
        let edge = TradingEdge {
            from_currency: market.currency_a.clone(),
            to_currency: market.currency_b.clone(),
            lowest_ratio: market.get_lowest_ratio().unwrap(),
            highest_ratio: market.get_highest_ratio().unwrap(),
            volume: market.volume_traded_currency_a,
        };
        // Safety: Already checked highest/lowest ratio for 0
        let reverse_edge = TradingEdge {
            from_currency: market.currency_b.clone(),
            to_currency: market.currency_a.clone(),
            lowest_ratio: 1.0 / market.get_highest_ratio().unwrap(),
            highest_ratio: 1.0 / market.get_lowest_ratio().unwrap(),
            volume: market.volume_traded_currency_b,
        };

        // Put both edges in the graph, since each market A|B is unique and there
        // is no market B|A, this way we get both edges.
        graph
            .entry(market.currency_a.clone())
            .or_default()
            .push(edge);
        graph
            .entry(market.currency_b.clone())
            .or_default()
            .push(reverse_edge);
    }
    graph
}

pub fn market_graph_dfs(
    graph: &Graph,
    start_hub: &TradingCurrencyType,
    max_depth: usize,
) -> Vec<Vec<TradingCurrencyType>> {
    // TODO: There's more short-circuiting logic that could go in here.
    let mut opportunities: Vec<Vec<&TradingCurrencyType>> = Vec::new();
    let mut stack: Vec<(&TradingCurrencyType, Vec<&TradingCurrencyType>)> = Vec::new();
    // We're going to use the tuple (&TCT, Vec<&TCT>) to track where we are
    // By re-traversing the given TCTs we can build the arbitrage path
    stack.push((start_hub, Vec::<&TradingCurrencyType>::new()));

    let hubs = [
        TradingCurrencyType::Divine,
        TradingCurrencyType::Chaos,
        TradingCurrencyType::Exalt,
    ];

    while let Some((current_vertex, mut current_path)) = stack.pop() {
        let Some(current_edges) = graph.get(current_vertex) else {
            continue;
        };

        // if we're not on a hub and we've seen this vertex before or there's some
        // issue with the edges, we skip
        if !hubs.contains(current_vertex)
            && (current_path.contains(&current_vertex) || current_edges.is_empty())
        {
            continue;
        }

        // We have something, stick it on the path
        current_path.push(current_vertex);

        // If we're too deep, we bail and move on to the next option
        if current_path.len() > max_depth {
            continue;
        }

        // If we have depth less than or equal to max_depth and we landed
        // on a different hub currency, we may have an arbitrage opportunity.
        // We can land on any other hub currency as the prices between them are (generally)
        // very stable.
        if current_path.len() <= max_depth
            && current_path.len() > 1
            && current_vertex != start_hub
            && hubs.contains(current_vertex)
        {
            // If we find an option we save it and then we're done
            opportunities.push(current_path.clone());
            continue;
        }

        // Next we have to go through all of the edges and put their connected
        // vertices on the stack, `to_currency` is the currency we're going to
        // next.
        let vertices: Vec<&TradingCurrencyType> =
            current_edges.iter().map(|e| &e.to_currency).collect();

        for v in vertices {
            stack.push((v, current_path.clone()));
        }
    }

    opportunities
        .into_iter()
        .map(|inner| inner.into_iter().cloned().collect())
        .collect()
}

/// Maybe get a reference to a specific edge from the graph, really just a convenience function
fn get_edge_from_graph<'a>(
    graph: &'a Graph,
    start_curr: &TradingCurrencyType,
    end_curr: &TradingCurrencyType,
) -> Option<&'a TradingEdge> {
    graph
        .get(start_curr)
        .and_then(|f| f.iter().find(|m| m.to_currency == *end_curr))
}

/// Calculate the return for a potential opportunity in the market
fn build_opportunity(opp: &[TradingCurrencyType], graph: &Graph) -> Option<ArbitrageOpportunity> {
    let mut high_ratios = Vec::new();
    let mut low_ratios = Vec::new();
    let mut volumes = Vec::new();

    // This could probably be written more idiomatically
    for window in opp.windows(2) {
        let from_cur = &window[0];
        let to_cur = &window[1];

        // If there's a problem getting an edge (shouldn't be) just give up
        let this_edge = get_edge_from_graph(graph, from_cur, to_cur)?;

        high_ratios.push(this_edge.highest_ratio);
        low_ratios.push(this_edge.lowest_ratio);
        volumes.push(this_edge.volume);
    }

    Some(ArbitrageOpportunity {
        path: opp.to_vec(),
        high_ratios,
        low_ratios,
        volumes,
    })
}

/// Determine if a given arbitrage opportunity is profitable
/// An opportunity is profitable if the path A -> X -> B -> A is more profitable
/// than just A -> B -> A and no step has zero volume.
/// Right now we only check if taking is profitable under the assumption that
/// there will be enough inefficiences that trying to eek out market making
/// profit is not worth our playtime.
pub fn arb_opp_is_profitable(
    arb_opp: &ArbitrageOpportunity,
    current_rates: &TradingCurrencyRates,
) -> bool {
    // Any more advanced filtering can be done on the front end, any computation
    // there should just be dividing volume steps by overall price for an estimate
    // on total trades.
    if arb_opp.volumes.contains(&0) {
        return false;
    }

    let final_opp_rate: f64 = arb_opp.high_ratios.iter().product();
    let Some(start_curr) = arb_opp.path.first() else {
        return false;
    };
    let Some(end_curr) = arb_opp.path.last() else {
        return false;
    };
    let comp_rate = match (start_curr, end_curr) {
        (TradingCurrencyType::Chaos, TradingCurrencyType::Exalt) => current_rates.chaos_to_exalt,
        (TradingCurrencyType::Divine, TradingCurrencyType::Exalt) => current_rates.div_to_exalt,
        (TradingCurrencyType::Divine, TradingCurrencyType::Chaos) => current_rates.div_to_chaos,
        (TradingCurrencyType::Chaos, TradingCurrencyType::Divine) => {
            1.0 / current_rates.div_to_chaos
        }
        (TradingCurrencyType::Exalt, TradingCurrencyType::Divine) => {
            1.0 / current_rates.div_to_exalt
        }
        (TradingCurrencyType::Exalt, TradingCurrencyType::Chaos) => {
            1.0 / current_rates.chaos_to_exalt
        }
        (_, _) => return false,
    };

    final_opp_rate > comp_rate
}

/// Runs a DFS over the markets to try and find open deals
pub fn get_all_arbitrage_options(graph: &Graph, max_depth: usize) -> Vec<ArbitrageOpportunity> {
    // TODO: Need some way to filter based on volume, too.

    let hubs = [
        TradingCurrencyType::Divine,
        TradingCurrencyType::Chaos,
        TradingCurrencyType::Exalt,
    ];

    hubs.iter()
        .flat_map(|hub| market_graph_dfs(graph, hub, max_depth))
        .filter(|opp| opp.len() == max_depth)
        .filter_map(|opp| build_opportunity(&opp, graph))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock Helper for building test markets
    fn mock_market(
        a: TradingCurrencyType,
        b: TradingCurrencyType,
        low: i64,
        high: i64,
        vol: i64,
    ) -> Market {
        Market {
            change_id: 1,
            league: "Test".to_string(),
            currency_a: a.clone(),
            currency_b: b.clone(),
            volume_traded_currency_a: vol,
            volume_traded_currency_b: vol * 10,
            lowest_stock_currency_a: 1000,
            lowest_stock_currency_b: 1000,
            highest_stock_currency_a: 2000,
            highest_stock_currency_b: 2000,
            lowest_ratio_currency_a: low * 2,
            lowest_ratio_currency_b: low,
            highest_ratio_currency_a: high * 5,
            highest_ratio_currency_b: high,
            is_hub_curr_a: a.is_hub(),
            is_hub_curr_b: b.is_hub(),
        }
    }

    #[test]
    fn test_graph_construction_and_inversion() {
        let markets = vec![mock_market(
            TradingCurrencyType::Chaos,
            TradingCurrencyType::Divine,
            100,
            500,
            100,
        )];

        let graph = build_and_populate_graph(&markets);

        // Assert Forward Edge exists
        let chaos_edges = graph.get(&TradingCurrencyType::Chaos).unwrap();
        assert_eq!(chaos_edges.len(), 1);
        assert_eq!(chaos_edges[0].to_currency, TradingCurrencyType::Divine);
        assert_eq!(chaos_edges[0].lowest_ratio, 2.);
        assert_eq!(chaos_edges[0].highest_ratio, 5.);

        // Assert Inverted Edge exists with inverted bounds
        let divine_edges = graph.get(&TradingCurrencyType::Divine).unwrap();
        assert_eq!(divine_edges.len(), 1);
        assert_eq!(divine_edges[0].to_currency, TradingCurrencyType::Chaos);
        assert!((divine_edges[0].lowest_ratio - (1.0 / 5.)).abs() < f64::EPSILON);
        assert!((divine_edges[0].highest_ratio - (1.0 / 2.)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_skips_zero_volume_markets() {
        let markets = vec![mock_market(
            TradingCurrencyType::Chaos,
            TradingCurrencyType::Divine,
            10,
            100,
            0,
        )];

        let graph = build_and_populate_graph(&markets);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_dfs_finds_simple_triangle_loop() {
        let mut graph: Graph = Graph::new();
        let chaos = TradingCurrencyType::Chaos;
        let divine = TradingCurrencyType::Divine;
        let test_item = TradingCurrencyType::Other("Test_Item".to_string());

        graph.entry(chaos.clone()).or_default().push(TradingEdge {
            from_currency: chaos.clone(),
            to_currency: test_item.clone(),
            lowest_ratio: 1.0,
            highest_ratio: 1.0,
            volume: 100,
        });

        graph
            .entry(test_item.clone())
            .or_default()
            .push(TradingEdge {
                from_currency: test_item.clone(),
                to_currency: chaos.clone(),
                lowest_ratio: 1.0,
                highest_ratio: 1.0,
                volume: 100,
            });

        graph
            .entry(test_item.clone())
            .or_default()
            .push(TradingEdge {
                from_currency: test_item.clone(),
                to_currency: divine.clone(),
                lowest_ratio: 1.0,
                highest_ratio: 1.0,
                volume: 100,
            });

        graph.entry(divine.clone()).or_default().push(TradingEdge {
            from_currency: divine.clone(),
            to_currency: test_item.clone(),
            lowest_ratio: 1.0,
            highest_ratio: 1.0,
            volume: 100,
        });

        let paths = market_graph_dfs(&graph, &chaos, 3);

        // Should find the path: Chaos -> Essence -> Chaos
        assert!(!paths.is_empty());
        assert_eq!(
            paths[0],
            vec![chaos.clone(), test_item.clone(), divine.clone(),]
        );
    }
}
