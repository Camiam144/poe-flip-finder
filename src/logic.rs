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
