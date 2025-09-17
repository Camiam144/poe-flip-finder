use crate::models::{ExchangeRecord, TradingCurrencyRates, TradingCurrencyType};

pub fn get_base_prices(records: &[ExchangeRecord], rates: &mut TradingCurrencyRates) {
    for record in records {
        let currency_pair = record.trading_currency();
        match currency_pair {
            (TradingCurrencyType::Divine, TradingCurrencyType::Exalt) => {
                rates.div_to_exalt = record.currency_one_data.relative_price
            }
            (TradingCurrencyType::Chaos, TradingCurrencyType::Exalt) => {
                rates.chaos_to_exalt = record.currency_one_data.relative_price
            }
            (_, _) => {}
        }
    }
    rates.div_to_chaos = rates.div_to_exalt / rates.chaos_to_exalt
}

// What do we have to do once we have the values?
// I think we're going to iterate over the filtered vector one time
// Maybe we can do it after the filter step.
// for each record we get the trading currency rate and we compare
// the absolute difference of that rate to the matching tradingcurrencyrate value.
// I think we then have another struct that's like {diff, ExchangeRecord} and
// push that into a new vec. Sort that vec by absolute difference, then we can
// pretty print the output? We need to compute the expected return at some point.
