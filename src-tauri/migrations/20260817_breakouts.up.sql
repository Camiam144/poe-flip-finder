CREATE TABLE IF NOT EXISTS poe2_markets (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	change_id INTEGER NOT NULL,
	league TEXT NOT NULL,
	market_id TEXT,
	currency_a_name_ggg TEXT,
	currency_b_name_ggg TEXT,
	currency_a_name_common TEXT,
	currency_b_name_common TEXT,
	volume_traded_currency_a INTEGER,
	volume_traded_currency_b INTEGER,
	lowest_stock_currency_a INTEGER,
	lowest_stock_currency_b INTEGER,
	highest_stock_currency_a INTEGER,
	highest_stock_currency_b INTEGER,
	lowest_ratio_currency_a INTEGER,
	lowest_ratio_currency_b INTEGER,
	highest_ratio_currency_a INTEGER,
	highest_ratio_currency_b INTEGER,
	is_hub_curr_a INTEGER,
	is_hub_curr_b INTEGER
);

CREATE TABLE IF NOT EXISTS poe1_markets (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	change_id INTEGER NOT NULL,
	league TEXT NOT NULL,
	market_id TEXT,
	currency_a_name_ggg TEXT,
	currency_b_name_ggg TEXT,
	currency_a_name_common TEXT,
	currency_b_name_common TEXT,
	volume_traded_currency_a INTEGER,
	volume_traded_currency_b INTEGER,
	lowest_stock_currency_a INTEGER,
	lowest_stock_currency_b INTEGER,
	highest_stock_currency_a INTEGER,
	highest_stock_currency_b INTEGER,
	lowest_ratio_currency_a INTEGER,
	lowest_ratio_currency_b INTEGER,
	highest_ratio_currency_a INTEGER,
	highest_ratio_currency_b INTEGER,
	is_hub_curr_a INTEGER,
	is_hub_curr_b INTEGER
);

ALTER TABLE data ADD COLUMN parsed_bool INTEGER;
