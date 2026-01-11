use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;
use csv::Writer;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "pivot-data-gen")]
#[command(about = "Generate synthetic trade data for pivot tables")]
pub struct Args {
    /// Number of rows to generate
    #[arg(short, long, default_value_t = 1000)]
    pub rows: usize,

    /// Number of portfolio managers
    #[arg(short, long, default_value_t = 10)]
    pub portfolio_managers: u32,

    /// Output file path for trades (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output file path for instruments reference data
    #[arg(long)]
    pub instruments_output: Option<PathBuf>,

    /// Output file path for constituents data
    #[arg(long)]
    pub constituents_output: Option<PathBuf>,

    /// Random seed for reproducibility
    #[arg(short, long, default_value_t = 42)]
    pub seed: u64,

    /// Trade date (YYYY-MM-DD format)
    #[arg(long, default_value = "2024-01-15")]
    pub trade_date: String,

    /// Explode ETF/ETC trades into constituent exposures
    #[arg(long, default_value_t = false)]
    pub explode_constituents: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Instrument {
    pub symbol: String,
    pub name: String,
    pub asset_class: String,
    pub instrument_type: String,
    pub currency: String,
    pub exchange: String,
    pub sector: String,
    pub is_composite: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Constituent {
    pub parent_symbol: String,
    pub constituent_symbol: String,
    pub weight: f64,
    pub shares_per_unit: f64,
    pub effective_date: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeRecord {
    pub trade_date: String,
    pub ts: String,
    pub portfolio_manager_id: u32,
    pub fund_id: u32,
    pub portfolio_id: u64,
    pub account_id: u64,
    pub desk: String,
    pub book: String,
    pub strategy: String,
    pub region: String,
    pub country: String,
    pub venue: String,
    pub asset_class: String,
    pub product: String,
    pub instrument_type: String,
    pub symbol: String,
    pub underlying_symbol: String,       // For constituents: the actual underlying (AAPL, GOLD, etc.)
    pub parent_symbol: String,           // For constituents: the parent ETF/ETC (SPY, GLD, etc.)
    pub exposure_type: String,           // "Direct", "ETF", "ETC", "Constituent"
    pub currency: String,
    pub counterparty: String,
    pub risk_bucket: String,
    pub scenario: String,
    pub trade_id: u64,
    pub order_id: u64,
    pub quantity: f64,
    pub price: f64,
    pub notional: f64,
    pub pnl: f64,
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
    pub margin: f64,
    pub fees: f64,
    pub slippage: f64,
    pub vol: f64,
    pub rate: f64,
    pub exposure: f64,
    pub weight: f64,
    pub metric_1: f64,
    pub metric_2: f64,
    pub metric_3: f64,
    pub metric_4: f64,
    pub metric_5: f64,
    pub metric_6: f64,
    pub metric_7: f64,
    pub metric_8: f64,
    pub metric_9: f64,
    pub metric_10: f64,
    pub metric_11: f64,
    pub metric_12: f64,
    pub metric_13: f64,
    pub metric_14: f64,
}

// Instrument definitions
fn build_instruments() -> Vec<Instrument> {
    vec![
        // Individual stocks
        Instrument { symbol: "AAPL".into(), name: "Apple Inc".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NASDAQ".into(), sector: "Technology".into(), is_composite: 0 },
        Instrument { symbol: "MSFT".into(), name: "Microsoft Corp".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NASDAQ".into(), sector: "Technology".into(), is_composite: 0 },
        Instrument { symbol: "GOOGL".into(), name: "Alphabet Inc".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NASDAQ".into(), sector: "Technology".into(), is_composite: 0 },
        Instrument { symbol: "AMZN".into(), name: "Amazon.com Inc".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NASDAQ".into(), sector: "Consumer".into(), is_composite: 0 },
        Instrument { symbol: "META".into(), name: "Meta Platforms Inc".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NASDAQ".into(), sector: "Technology".into(), is_composite: 0 },
        Instrument { symbol: "NVDA".into(), name: "NVIDIA Corp".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NASDAQ".into(), sector: "Technology".into(), is_composite: 0 },
        Instrument { symbol: "TSLA".into(), name: "Tesla Inc".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NASDAQ".into(), sector: "Consumer".into(), is_composite: 0 },
        Instrument { symbol: "JPM".into(), name: "JPMorgan Chase".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Financials".into(), is_composite: 0 },
        Instrument { symbol: "V".into(), name: "Visa Inc".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Financials".into(), is_composite: 0 },
        Instrument { symbol: "JNJ".into(), name: "Johnson & Johnson".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Healthcare".into(), is_composite: 0 },
        Instrument { symbol: "WMT".into(), name: "Walmart Inc".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Consumer".into(), is_composite: 0 },
        Instrument { symbol: "XOM".into(), name: "Exxon Mobil".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Energy".into(), is_composite: 0 },
        Instrument { symbol: "BAC".into(), name: "Bank of America".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Financials".into(), is_composite: 0 },
        Instrument { symbol: "PG".into(), name: "Procter & Gamble".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Consumer".into(), is_composite: 0 },
        Instrument { symbol: "HD".into(), name: "Home Depot".into(), asset_class: "Equity".into(), instrument_type: "Stock".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Consumer".into(), is_composite: 0 },
        // Commodities for ETCs
        Instrument { symbol: "GOLD".into(), name: "Gold Spot".into(), asset_class: "Commodity".into(), instrument_type: "Commodity".into(), currency: "USD".into(), exchange: "COMEX".into(), sector: "Precious Metals".into(), is_composite: 0 },
        Instrument { symbol: "SILVER".into(), name: "Silver Spot".into(), asset_class: "Commodity".into(), instrument_type: "Commodity".into(), currency: "USD".into(), exchange: "COMEX".into(), sector: "Precious Metals".into(), is_composite: 0 },
        Instrument { symbol: "PLAT".into(), name: "Platinum Spot".into(), asset_class: "Commodity".into(), instrument_type: "Commodity".into(), currency: "USD".into(), exchange: "COMEX".into(), sector: "Precious Metals".into(), is_composite: 0 },
        Instrument { symbol: "COPPER".into(), name: "Copper".into(), asset_class: "Commodity".into(), instrument_type: "Commodity".into(), currency: "USD".into(), exchange: "COMEX".into(), sector: "Industrial Metals".into(), is_composite: 0 },
        Instrument { symbol: "CRUDEOIL".into(), name: "Crude Oil WTI".into(), asset_class: "Commodity".into(), instrument_type: "Commodity".into(), currency: "USD".into(), exchange: "NYMEX".into(), sector: "Energy".into(), is_composite: 0 },
        Instrument { symbol: "NATGAS".into(), name: "Natural Gas".into(), asset_class: "Commodity".into(), instrument_type: "Commodity".into(), currency: "USD".into(), exchange: "NYMEX".into(), sector: "Energy".into(), is_composite: 0 },
        // ETFs (composite instruments)
        Instrument { symbol: "SPY".into(), name: "SPDR S&P 500 ETF".into(), asset_class: "Equity".into(), instrument_type: "ETF".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Broad Market".into(), is_composite: 1 },
        Instrument { symbol: "QQQ".into(), name: "Invesco QQQ Trust".into(), asset_class: "Equity".into(), instrument_type: "ETF".into(), currency: "USD".into(), exchange: "NASDAQ".into(), sector: "Technology".into(), is_composite: 1 },
        Instrument { symbol: "XLF".into(), name: "Financial Select SPDR".into(), asset_class: "Equity".into(), instrument_type: "ETF".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Financials".into(), is_composite: 1 },
        Instrument { symbol: "XLE".into(), name: "Energy Select SPDR".into(), asset_class: "Equity".into(), instrument_type: "ETF".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Energy".into(), is_composite: 1 },
        // ETCs (commodity ETFs)
        Instrument { symbol: "GLD".into(), name: "SPDR Gold Shares".into(), asset_class: "Commodity".into(), instrument_type: "ETC".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Precious Metals".into(), is_composite: 1 },
        Instrument { symbol: "SLV".into(), name: "iShares Silver Trust".into(), asset_class: "Commodity".into(), instrument_type: "ETC".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Precious Metals".into(), is_composite: 1 },
        Instrument { symbol: "USO".into(), name: "United States Oil Fund".into(), asset_class: "Commodity".into(), instrument_type: "ETC".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Energy".into(), is_composite: 1 },
        Instrument { symbol: "PMET".into(), name: "Precious Metals Basket ETC".into(), asset_class: "Commodity".into(), instrument_type: "ETC".into(), currency: "USD".into(), exchange: "NYSE".into(), sector: "Precious Metals".into(), is_composite: 1 },
    ]
}

// Constituent mappings
fn build_constituents(effective_date: &str) -> Vec<Constituent> {
    vec![
        // SPY - S&P 500 ETF (simplified top holdings)
        Constituent { parent_symbol: "SPY".into(), constituent_symbol: "AAPL".into(), weight: 0.07, shares_per_unit: 0.45, effective_date: effective_date.into() },
        Constituent { parent_symbol: "SPY".into(), constituent_symbol: "MSFT".into(), weight: 0.065, shares_per_unit: 0.25, effective_date: effective_date.into() },
        Constituent { parent_symbol: "SPY".into(), constituent_symbol: "GOOGL".into(), weight: 0.04, shares_per_unit: 0.30, effective_date: effective_date.into() },
        Constituent { parent_symbol: "SPY".into(), constituent_symbol: "AMZN".into(), weight: 0.035, shares_per_unit: 0.22, effective_date: effective_date.into() },
        Constituent { parent_symbol: "SPY".into(), constituent_symbol: "NVDA".into(), weight: 0.03, shares_per_unit: 0.15, effective_date: effective_date.into() },
        Constituent { parent_symbol: "SPY".into(), constituent_symbol: "META".into(), weight: 0.025, shares_per_unit: 0.08, effective_date: effective_date.into() },
        Constituent { parent_symbol: "SPY".into(), constituent_symbol: "TSLA".into(), weight: 0.02, shares_per_unit: 0.10, effective_date: effective_date.into() },
        Constituent { parent_symbol: "SPY".into(), constituent_symbol: "JPM".into(), weight: 0.015, shares_per_unit: 0.12, effective_date: effective_date.into() },
        // QQQ - Nasdaq 100 ETF
        Constituent { parent_symbol: "QQQ".into(), constituent_symbol: "AAPL".into(), weight: 0.12, shares_per_unit: 0.78, effective_date: effective_date.into() },
        Constituent { parent_symbol: "QQQ".into(), constituent_symbol: "MSFT".into(), weight: 0.10, shares_per_unit: 0.40, effective_date: effective_date.into() },
        Constituent { parent_symbol: "QQQ".into(), constituent_symbol: "GOOGL".into(), weight: 0.08, shares_per_unit: 0.60, effective_date: effective_date.into() },
        Constituent { parent_symbol: "QQQ".into(), constituent_symbol: "AMZN".into(), weight: 0.07, shares_per_unit: 0.45, effective_date: effective_date.into() },
        Constituent { parent_symbol: "QQQ".into(), constituent_symbol: "NVDA".into(), weight: 0.06, shares_per_unit: 0.30, effective_date: effective_date.into() },
        Constituent { parent_symbol: "QQQ".into(), constituent_symbol: "META".into(), weight: 0.05, shares_per_unit: 0.16, effective_date: effective_date.into() },
        Constituent { parent_symbol: "QQQ".into(), constituent_symbol: "TSLA".into(), weight: 0.04, shares_per_unit: 0.20, effective_date: effective_date.into() },
        // XLF - Financials ETF
        Constituent { parent_symbol: "XLF".into(), constituent_symbol: "JPM".into(), weight: 0.10, shares_per_unit: 0.25, effective_date: effective_date.into() },
        Constituent { parent_symbol: "XLF".into(), constituent_symbol: "BAC".into(), weight: 0.08, shares_per_unit: 2.50, effective_date: effective_date.into() },
        Constituent { parent_symbol: "XLF".into(), constituent_symbol: "V".into(), weight: 0.06, shares_per_unit: 0.25, effective_date: effective_date.into() },
        // XLE - Energy ETF
        Constituent { parent_symbol: "XLE".into(), constituent_symbol: "XOM".into(), weight: 0.22, shares_per_unit: 0.90, effective_date: effective_date.into() },
        // GLD - Gold ETC (backed by physical gold)
        Constituent { parent_symbol: "GLD".into(), constituent_symbol: "GOLD".into(), weight: 1.0, shares_per_unit: 0.093, effective_date: effective_date.into() },
        // SLV - Silver ETC
        Constituent { parent_symbol: "SLV".into(), constituent_symbol: "SILVER".into(), weight: 1.0, shares_per_unit: 0.92, effective_date: effective_date.into() },
        // USO - Oil ETC
        Constituent { parent_symbol: "USO".into(), constituent_symbol: "CRUDEOIL".into(), weight: 1.0, shares_per_unit: 0.80, effective_date: effective_date.into() },
        // PMET - Precious Metals Basket
        Constituent { parent_symbol: "PMET".into(), constituent_symbol: "GOLD".into(), weight: 0.50, shares_per_unit: 0.05, effective_date: effective_date.into() },
        Constituent { parent_symbol: "PMET".into(), constituent_symbol: "SILVER".into(), weight: 0.30, shares_per_unit: 0.50, effective_date: effective_date.into() },
        Constituent { parent_symbol: "PMET".into(), constituent_symbol: "PLAT".into(), weight: 0.20, shares_per_unit: 0.02, effective_date: effective_date.into() },
    ]
}

pub struct DataGenerator {
    rng: StdRng,
    trade_date: NaiveDate,
    portfolio_managers: u32,
    trade_counter: u64,
    order_counter: u64,
    instruments: Vec<Instrument>,
    constituents: Vec<Constituent>,
    constituent_map: HashMap<String, Vec<Constituent>>,
    explode_constituents: bool,
}

const DESKS: &[&str] = &["Equities", "FixedIncome", "Commodities", "FX", "Derivatives"];
const BOOKS: &[&str] = &["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta"];
const STRATEGIES: &[&str] = &["Momentum", "MeanReversion", "StatArb", "EventDriven", "Macro"];
const REGIONS: &[&str] = &["AMER", "EMEA", "APAC"];
const COUNTRIES: &[&str] = &["US", "UK", "DE", "JP", "HK", "SG", "AU", "FR", "CH", "CA"];
const COUNTERPARTIES: &[&str] = &[
    "GoldmanSachs", "JPMorgan", "MorganStanley", "Citadel", "TwoSigma",
    "Bridgewater", "BlackRock", "Vanguard", "StateStreet", "Fidelity",
];
const RISK_BUCKETS: &[&str] = &["Low", "Medium", "High", "VeryHigh"];
const SCENARIOS: &[&str] = &["Base", "Stress", "Historical", "MonteCarlo"];

impl DataGenerator {
    pub fn new(seed: u64, trade_date: NaiveDate, portfolio_managers: u32, explode_constituents: bool) -> Self {
        let instruments = build_instruments();
        let constituents = build_constituents(&trade_date.format("%Y-%m-%d").to_string());

        let mut constituent_map: HashMap<String, Vec<Constituent>> = HashMap::new();
        for c in &constituents {
            constituent_map
                .entry(c.parent_symbol.clone())
                .or_default()
                .push(c.clone());
        }

        Self {
            rng: StdRng::seed_from_u64(seed),
            trade_date,
            portfolio_managers,
            trade_counter: 0,
            order_counter: 0,
            instruments,
            constituents,
            constituent_map,
            explode_constituents,
        }
    }

    pub fn instruments(&self) -> &[Instrument] {
        &self.instruments
    }

    pub fn constituents(&self) -> &[Constituent] {
        &self.constituents
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.rng.gen_range(0..items.len())]
    }

    fn pick_instrument(&mut self) -> &Instrument {
        &self.instruments[self.rng.gen_range(0..self.instruments.len())]
    }

    fn gen_timestamp(&mut self) -> NaiveDateTime {
        let hour = self.rng.gen_range(9..17);
        let minute = self.rng.gen_range(0..60);
        let second = self.rng.gen_range(0..60);
        let milli = self.rng.gen_range(0..1000);
        self.trade_date
            .and_hms_milli_opt(hour, minute, second, milli)
            .unwrap()
    }

    fn gen_metrics(&mut self) -> [f64; 14] {
        [
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
            self.rng.gen_range(-100.0..100.0),
        ]
    }

    pub fn generate_records(&mut self) -> Vec<TradeRecord> {
        self.trade_counter += 1;
        if self.trade_counter % 5 == 1 {
            self.order_counter += 1;
        }

        let instrument = self.pick_instrument().clone();
        let pm_id = self.rng.gen_range(1..=self.portfolio_managers);
        let fund_id = self.rng.gen_range(1..=50);
        let quantity = self.rng.gen_range(100.0..10000.0);
        let price = self.rng.gen_range(10.0..500.0);
        let notional = quantity * price;
        let pnl = self.rng.gen_range(-50000.0..50000.0);
        let ts = self.gen_timestamp();

        let desk = self.pick(DESKS).to_string();
        let book = self.pick(BOOKS).to_string();
        let strategy = self.pick(STRATEGIES).to_string();
        let region = self.pick(REGIONS).to_string();
        let country = self.pick(COUNTRIES).to_string();
        let counterparty = self.pick(COUNTERPARTIES).to_string();
        let risk_bucket = self.pick(RISK_BUCKETS).to_string();
        let scenario = self.pick(SCENARIOS).to_string();

        // Determine exposure_type based on instrument
        let exposure_type = if instrument.is_composite == 1 {
            instrument.instrument_type.clone() // "ETF" or "ETC"
        } else {
            "Direct".to_string()
        };

        let base_record = TradeRecord {
            trade_date: self.trade_date.format("%Y-%m-%d").to_string(),
            ts: ts.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            portfolio_manager_id: pm_id,
            fund_id,
            portfolio_id: (pm_id as u64) * 1000 + self.rng.gen_range(1..100),
            account_id: (fund_id as u64) * 10000 + self.rng.gen_range(1..1000),
            desk: desk.clone(),
            book: book.clone(),
            strategy: strategy.clone(),
            region: region.clone(),
            country: country.clone(),
            venue: instrument.exchange.clone(),
            asset_class: instrument.asset_class.clone(),
            product: if instrument.instrument_type == "Stock" { "Spot".into() } else { instrument.instrument_type.clone() },
            instrument_type: instrument.instrument_type.clone(),
            symbol: instrument.symbol.clone(),
            underlying_symbol: instrument.symbol.clone(), // For direct/ETF/ETC, symbol is the underlying
            parent_symbol: String::new(),                 // No parent for top-level trades
            exposure_type,
            currency: instrument.currency.clone(),
            counterparty: counterparty.clone(),
            risk_bucket: risk_bucket.clone(),
            scenario: scenario.clone(),
            trade_id: self.trade_counter,
            order_id: self.order_counter,
            quantity,
            price,
            notional,
            pnl,
            delta: self.rng.gen_range(-1.0..1.0),
            gamma: self.rng.gen_range(0.0..0.1),
            vega: self.rng.gen_range(-100.0..100.0),
            theta: self.rng.gen_range(-50.0..0.0),
            rho: self.rng.gen_range(-10.0..10.0),
            margin: notional * self.rng.gen_range(0.05..0.20),
            fees: notional * self.rng.gen_range(0.0001..0.001),
            slippage: self.rng.gen_range(0.0..0.005),
            vol: self.rng.gen_range(0.1..0.8),
            rate: self.rng.gen_range(0.01..0.05),
            exposure: notional * self.rng.gen_range(0.5..1.5),
            weight: 1.0,
            metric_1: self.rng.gen_range(-100.0..100.0),
            metric_2: self.rng.gen_range(-100.0..100.0),
            metric_3: self.rng.gen_range(-100.0..100.0),
            metric_4: self.rng.gen_range(-100.0..100.0),
            metric_5: self.rng.gen_range(-100.0..100.0),
            metric_6: self.rng.gen_range(-100.0..100.0),
            metric_7: self.rng.gen_range(-100.0..100.0),
            metric_8: self.rng.gen_range(-100.0..100.0),
            metric_9: self.rng.gen_range(-100.0..100.0),
            metric_10: self.rng.gen_range(-100.0..100.0),
            metric_11: self.rng.gen_range(-100.0..100.0),
            metric_12: self.rng.gen_range(-100.0..100.0),
            metric_13: self.rng.gen_range(-100.0..100.0),
            metric_14: self.rng.gen_range(-100.0..100.0),
        };

        let mut records = vec![base_record.clone()];

        // If this is a composite instrument and we're exploding constituents
        if self.explode_constituents && instrument.is_composite == 1 {
            let constituents = self.constituent_map.get(&instrument.symbol).cloned();
            if let Some(constituents) = constituents {
                for c in &constituents {
                    let constituent_qty = quantity * c.shares_per_unit;
                    let constituent_notional = notional * c.weight;
                    let constituent_pnl = pnl * c.weight;
                    let metrics = self.gen_metrics();

                    records.push(TradeRecord {
                        trade_date: base_record.trade_date.clone(),
                        ts: base_record.ts.clone(),
                        portfolio_manager_id: pm_id,
                        fund_id,
                        portfolio_id: base_record.portfolio_id,
                        account_id: base_record.account_id,
                        desk: desk.clone(),
                        book: book.clone(),
                        strategy: strategy.clone(),
                        region: region.clone(),
                        country: country.clone(),
                        venue: base_record.venue.clone(),
                        asset_class: base_record.asset_class.clone(),
                        product: "Constituent".into(),
                        instrument_type: "Constituent".into(),
                        symbol: instrument.symbol.clone(),           // Keep parent symbol for grouping
                        underlying_symbol: c.constituent_symbol.clone(), // The actual underlying
                        parent_symbol: instrument.symbol.clone(),    // Link back to parent ETF/ETC
                        exposure_type: "Constituent".to_string(),    // Mark as constituent exposure
                        currency: base_record.currency.clone(),
                        counterparty: counterparty.clone(),
                        risk_bucket: risk_bucket.clone(),
                        scenario: scenario.clone(),
                        trade_id: self.trade_counter,
                        order_id: self.order_counter,
                        quantity: constituent_qty,
                        price: price * c.weight,
                        notional: constituent_notional,
                        pnl: constituent_pnl,
                        delta: base_record.delta * c.weight,
                        gamma: base_record.gamma * c.weight,
                        vega: base_record.vega * c.weight,
                        theta: base_record.theta * c.weight,
                        rho: base_record.rho * c.weight,
                        margin: base_record.margin * c.weight,
                        fees: base_record.fees * c.weight,
                        slippage: base_record.slippage,
                        vol: base_record.vol,
                        rate: base_record.rate,
                        exposure: base_record.exposure * c.weight,
                        weight: c.weight,
                        metric_1: metrics[0],
                        metric_2: metrics[1],
                        metric_3: metrics[2],
                        metric_4: metrics[3],
                        metric_5: metrics[4],
                        metric_6: metrics[5],
                        metric_7: metrics[6],
                        metric_8: metrics[7],
                        metric_9: metrics[8],
                        metric_10: metrics[9],
                        metric_11: metrics[10],
                        metric_12: metrics[11],
                        metric_13: metrics[12],
                        metric_14: metrics[13],
                    });
                }
            }
        }

        records
    }
}

pub struct GenerationResult {
    pub rows_generated: usize,
    pub duration_ms: u128,
    pub bytes_written: usize,
    pub constituent_rows: usize,
}

pub fn generate_to_file(args: &Args) -> Result<GenerationResult, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let trade_date = NaiveDate::parse_from_str(&args.trade_date, "%Y-%m-%d")?;
    let mut generator = DataGenerator::new(args.seed, trade_date, args.portfolio_managers, args.explode_constituents);

    let mut total_rows = 0;
    let mut constituent_rows = 0;

    // Generate instruments file if requested
    if let Some(path) = &args.instruments_output {
        let file = File::create(path)?;
        let mut csv_writer = Writer::from_writer(file);
        for instrument in generator.instruments() {
            csv_writer.serialize(instrument)?;
        }
        csv_writer.flush()?;
    }

    // Generate constituents file if requested
    if let Some(path) = &args.constituents_output {
        let file = File::create(path)?;
        let mut csv_writer = Writer::from_writer(file);
        for constituent in generator.constituents() {
            csv_writer.serialize(constituent)?;
        }
        csv_writer.flush()?;
    }

    // Generate trades
    match &args.output {
        Some(path) => {
            let file = File::create(path)?;
            let mut csv_writer = Writer::from_writer(file);

            for _ in 0..args.rows {
                let records = generator.generate_records();
                for record in &records {
                    if record.exposure_type == "Constituent" {
                        constituent_rows += 1;
                    }
                    csv_writer.serialize(record)?;
                    total_rows += 1;
                }
            }

            csv_writer.flush()?;
            let bytes = path.metadata().map(|m| m.len() as usize).unwrap_or(0);

            Ok(GenerationResult {
                rows_generated: total_rows,
                duration_ms: start.elapsed().as_millis(),
                bytes_written: bytes,
                constituent_rows,
            })
        }
        None => {
            let mut csv_writer = Writer::from_writer(io::stdout());

            for _ in 0..args.rows {
                let records = generator.generate_records();
                for record in &records {
                    if record.exposure_type == "Constituent" {
                        constituent_rows += 1;
                    }
                    csv_writer.serialize(record)?;
                    total_rows += 1;
                }
            }

            csv_writer.flush()?;

            Ok(GenerationResult {
                rows_generated: total_rows,
                duration_ms: start.elapsed().as_millis(),
                bytes_written: 0,
                constituent_rows,
            })
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let result = generate_to_file(&args)?;

    eprintln!("Generated {} rows in {}ms", result.rows_generated, result.duration_ms);
    if result.constituent_rows > 0 {
        eprintln!("  - {} constituent exposure rows", result.constituent_rows);
    }
    if result.bytes_written > 0 {
        eprintln!("Output size: {} bytes", result.bytes_written);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_single_record() {
        let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut generator = DataGenerator::new(42, trade_date, 1, false);
        let records = generator.generate_records();

        assert!(!records.is_empty());
        let record = &records[0];
        assert_eq!(record.trade_date, "2024-01-15");
        assert_eq!(record.portfolio_manager_id, 1);
        assert!(record.quantity > 0.0);
        assert!(record.price > 0.0);
    }

    #[test]
    fn test_generate_20_rows_1_pm_50_columns() {
        let args = Args {
            rows: 20,
            portfolio_managers: 1,
            output: None,
            instruments_output: None,
            constituents_output: None,
            seed: 42,
            trade_date: "2024-01-15".to_string(),
            explode_constituents: false,
        };

        let mut buffer = Cursor::new(Vec::new());
        let trade_date = NaiveDate::parse_from_str(&args.trade_date, "%Y-%m-%d").unwrap();
        let mut generator = DataGenerator::new(args.seed, trade_date, args.portfolio_managers, false);

        {
            let mut csv_writer = Writer::from_writer(&mut buffer);
            for _ in 0..args.rows {
                let records = generator.generate_records();
                for record in records {
                    csv_writer.serialize(&record).unwrap();
                }
            }
            csv_writer.flush().unwrap();
        }

        let csv_content = String::from_utf8(buffer.into_inner()).unwrap();
        let lines: Vec<&str> = csv_content.lines().collect();

        // Header + 20 data rows
        assert_eq!(lines.len(), 21, "Expected 21 lines (1 header + 20 rows)");

        // Verify column count in header (55 columns with exposure_type support)
        let header_cols: Vec<&str> = lines[0].split(',').collect();
        assert_eq!(header_cols.len(), 55, "Expected 55 columns");

        // Verify all rows have portfolio_manager_id = 1
        for line in &lines[1..] {
            let cols: Vec<&str> = line.split(',').collect();
            assert_eq!(cols[2], "1", "portfolio_manager_id should be 1");
        }
    }

    #[test]
    fn test_constituent_explosion() {
        let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut generator = DataGenerator::new(100, trade_date, 1, true);

        // Generate until we hit an ETF
        let mut found_constituents = false;
        for _ in 0..100 {
            let records = generator.generate_records();
            if records.len() > 1 {
                // Found a composite instrument
                found_constituents = true;
                let parent = &records[0];
                assert!(parent.exposure_type == "ETF" || parent.exposure_type == "ETC");

                for constituent in &records[1..] {
                    assert_eq!(constituent.exposure_type, "Constituent");
                    assert_eq!(constituent.parent_symbol, parent.symbol);
                    assert!(!constituent.underlying_symbol.is_empty());
                    assert!(constituent.weight > 0.0 && constituent.weight <= 1.0);
                }
                break;
            }
        }
        assert!(found_constituents, "Should have found at least one composite instrument");
    }

    #[test]
    fn test_deterministic_output_with_seed() {
        let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let mut gen1 = DataGenerator::new(123, trade_date, 5, false);
        let mut gen2 = DataGenerator::new(123, trade_date, 5, false);

        for _ in 0..10 {
            let r1 = &gen1.generate_records()[0];
            let r2 = &gen2.generate_records()[0];
            assert_eq!(r1.trade_id, r2.trade_id);
            assert_eq!(r1.symbol, r2.symbol);
            assert_eq!(r1.quantity, r2.quantity);
        }
    }

    #[test]
    fn test_output_to_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let args = Args {
            rows: 10,
            portfolio_managers: 2,
            output: Some(path.clone()),
            instruments_output: None,
            constituents_output: None,
            seed: 42,
            trade_date: "2024-01-15".to_string(),
            explode_constituents: false,
        };

        let result = generate_to_file(&args).unwrap();

        assert_eq!(result.rows_generated, 10);
        assert!(result.duration_ms < 1000);
        assert!(result.bytes_written > 0);

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 11); // header + 10 rows
    }

    #[test]
    fn test_instruments_and_constituents_output() {
        let temp_instruments = NamedTempFile::new().unwrap();
        let temp_constituents = NamedTempFile::new().unwrap();
        let temp_trades = NamedTempFile::new().unwrap();

        let args = Args {
            rows: 5,
            portfolio_managers: 1,
            output: Some(temp_trades.path().to_path_buf()),
            instruments_output: Some(temp_instruments.path().to_path_buf()),
            constituents_output: Some(temp_constituents.path().to_path_buf()),
            seed: 42,
            trade_date: "2024-01-15".to_string(),
            explode_constituents: false,
        };

        generate_to_file(&args).unwrap();

        // Check instruments file
        let instruments_content = std::fs::read_to_string(temp_instruments.path()).unwrap();
        assert!(instruments_content.contains("SPY"));
        assert!(instruments_content.contains("GLD"));
        assert!(instruments_content.contains("AAPL"));

        // Check constituents file
        let constituents_content = std::fs::read_to_string(temp_constituents.path()).unwrap();
        assert!(constituents_content.contains("SPY,AAPL"));
        assert!(constituents_content.contains("GLD,GOLD"));
    }

    #[test]
    fn test_greeks_in_valid_ranges() {
        let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut generator = DataGenerator::new(42, trade_date, 1, false);

        for _ in 0..100 {
            let record = &generator.generate_records()[0];
            assert!(record.delta >= -1.0 && record.delta <= 1.0);
            assert!(record.gamma >= 0.0 && record.gamma <= 0.1);
            assert!(record.theta <= 0.0);
        }
    }
}
