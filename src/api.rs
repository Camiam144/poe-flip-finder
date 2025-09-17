use reqwest::blocking;

use crate::models::ExchangeSnapshot;

pub fn get_exchange_snapshot() -> reqwest::Result<ExchangeSnapshot> {
    let url = "https://poe2scout.com/api/currencyExchangeSnapshot?league=Rise%20of%20the%20Abyssal";
    blocking::get(url)?.json::<ExchangeSnapshot>()
}
