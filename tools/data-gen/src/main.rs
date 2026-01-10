use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;
use csv::Writer;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
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

    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Random seed for reproducibility
    #[arg(short, long, default_value_t = 42)]
    pub seed: u64,

    /// Trade date (YYYY-MM-DD format)
    #[arg(long, default_value = "2024-01-15")]
    pub trade_date: String,
}

#[derive(Debug, Serialize)]
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
    pub symbol: String,
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

pub struct DataGenerator {
    rng: StdRng,
    trade_date: NaiveDate,
    portfolio_managers: u32,
    trade_counter: u64,
    order_counter: u64,
}

const DESKS: &[&str] = &["Equities", "FixedIncome", "Commodities", "FX", "Derivatives"];
const BOOKS: &[&str] = &["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta"];
const STRATEGIES: &[&str] = &["Momentum", "MeanReversion", "StatArb", "EventDriven", "Macro"];
const REGIONS: &[&str] = &["AMER", "EMEA", "APAC"];
const COUNTRIES: &[&str] = &["US", "UK", "DE", "JP", "HK", "SG", "AU", "FR", "CH", "CA"];
const VENUES: &[&str] = &["NYSE", "NASDAQ", "LSE", "XETRA", "TSE", "HKEX", "SGX"];
const ASSET_CLASSES: &[&str] = &["Equity", "Bond", "Future", "Option", "Swap", "FX"];
const PRODUCTS: &[&str] = &["Spot", "Forward", "Swap", "Option", "Future"];
const SYMBOLS: &[&str] = &[
    "AAPL", "MSFT", "GOOGL", "AMZN", "META", "NVDA", "TSLA", "JPM", "V", "JNJ",
    "WMT", "PG", "MA", "UNH", "HD", "DIS", "BAC", "XOM", "PFE", "KO",
];
const CURRENCIES: &[&str] = &["USD", "EUR", "GBP", "JPY", "CHF", "AUD", "CAD", "HKD", "SGD"];
const COUNTERPARTIES: &[&str] = &[
    "GoldmanSachs", "JPMorgan", "MorganStanley", "Citadel", "TwoSigma",
    "Bridgewater", "BlackRock", "Vanguard", "StateStreet", "Fidelity",
];
const RISK_BUCKETS: &[&str] = &["Low", "Medium", "High", "VeryHigh"];
const SCENARIOS: &[&str] = &["Base", "Stress", "Historical", "MonteCarlo"];

impl DataGenerator {
    pub fn new(seed: u64, trade_date: NaiveDate, portfolio_managers: u32) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            trade_date,
            portfolio_managers,
            trade_counter: 0,
            order_counter: 0,
        }
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.rng.gen_range(0..items.len())]
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

    pub fn generate_record(&mut self) -> TradeRecord {
        self.trade_counter += 1;
        if self.trade_counter % 5 == 1 {
            self.order_counter += 1;
        }

        let pm_id = self.rng.gen_range(1..=self.portfolio_managers);
        let fund_id = self.rng.gen_range(1..=50);
        let quantity = self.rng.gen_range(100.0..10000.0);
        let price = self.rng.gen_range(10.0..500.0);
        let notional = quantity * price;

        TradeRecord {
            trade_date: self.trade_date.format("%Y-%m-%d").to_string(),
            ts: self.gen_timestamp().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            portfolio_manager_id: pm_id,
            fund_id,
            portfolio_id: (pm_id as u64) * 1000 + self.rng.gen_range(1..100),
            account_id: (fund_id as u64) * 10000 + self.rng.gen_range(1..1000),
            desk: self.pick(DESKS).to_string(),
            book: self.pick(BOOKS).to_string(),
            strategy: self.pick(STRATEGIES).to_string(),
            region: self.pick(REGIONS).to_string(),
            country: self.pick(COUNTRIES).to_string(),
            venue: self.pick(VENUES).to_string(),
            asset_class: self.pick(ASSET_CLASSES).to_string(),
            product: self.pick(PRODUCTS).to_string(),
            symbol: self.pick(SYMBOLS).to_string(),
            currency: self.pick(CURRENCIES).to_string(),
            counterparty: self.pick(COUNTERPARTIES).to_string(),
            risk_bucket: self.pick(RISK_BUCKETS).to_string(),
            scenario: self.pick(SCENARIOS).to_string(),
            trade_id: self.trade_counter,
            order_id: self.order_counter,
            quantity,
            price,
            notional,
            pnl: self.rng.gen_range(-50000.0..50000.0),
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
        }
    }
}

pub struct GenerationResult {
    pub rows_generated: usize,
    pub duration_ms: u128,
    pub bytes_written: usize,
}

pub fn generate_to_vec(args: &Args) -> Result<(Vec<u8>, GenerationResult), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let trade_date = NaiveDate::parse_from_str(&args.trade_date, "%Y-%m-%d")?;
    let mut generator = DataGenerator::new(args.seed, trade_date, args.portfolio_managers);
    let mut csv_writer = Writer::from_writer(Vec::new());

    for _ in 0..args.rows {
        let record = generator.generate_record();
        csv_writer.serialize(&record)?;
    }

    csv_writer.flush()?;
    let data = csv_writer.into_inner()?;
    let bytes = data.len();

    Ok((data, GenerationResult {
        rows_generated: args.rows,
        duration_ms: start.elapsed().as_millis(),
        bytes_written: bytes,
    }))
}

pub fn generate_to_file(args: &Args) -> Result<GenerationResult, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let trade_date = NaiveDate::parse_from_str(&args.trade_date, "%Y-%m-%d")?;
    let mut generator = DataGenerator::new(args.seed, trade_date, args.portfolio_managers);

    match &args.output {
        Some(path) => {
            let file = File::create(path)?;
            let mut csv_writer = Writer::from_writer(file);

            for _ in 0..args.rows {
                let record = generator.generate_record();
                csv_writer.serialize(&record)?;
            }

            csv_writer.flush()?;
            let bytes = path.metadata().map(|m| m.len() as usize).unwrap_or(0);

            Ok(GenerationResult {
                rows_generated: args.rows,
                duration_ms: start.elapsed().as_millis(),
                bytes_written: bytes,
            })
        }
        None => {
            let mut csv_writer = Writer::from_writer(io::stdout());

            for _ in 0..args.rows {
                let record = generator.generate_record();
                csv_writer.serialize(&record)?;
            }

            csv_writer.flush()?;

            Ok(GenerationResult {
                rows_generated: args.rows,
                duration_ms: start.elapsed().as_millis(),
                bytes_written: 0,
            })
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let result = generate_to_file(&args)?;

    eprintln!("Generated {} rows in {}ms", result.rows_generated, result.duration_ms);
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
        let mut generator = DataGenerator::new(42, trade_date, 1);
        let record = generator.generate_record();

        assert_eq!(record.trade_date, "2024-01-15");
        assert_eq!(record.portfolio_manager_id, 1);
        assert!(record.quantity > 0.0);
        assert!(record.price > 0.0);
        assert!((record.notional - record.quantity * record.price).abs() < 0.01);
    }

    #[test]
    fn test_generate_20_rows_1_pm_50_columns() {
        let args = Args {
            rows: 20,
            portfolio_managers: 1,
            output: None,
            seed: 42,
            trade_date: "2024-01-15".to_string(),
        };

        let mut buffer = Cursor::new(Vec::new());
        let trade_date = NaiveDate::parse_from_str(&args.trade_date, "%Y-%m-%d").unwrap();
        let mut generator = DataGenerator::new(args.seed, trade_date, args.portfolio_managers);

        {
            let mut csv_writer = Writer::from_writer(&mut buffer);
            for _ in 0..args.rows {
                let record = generator.generate_record();
                csv_writer.serialize(&record).unwrap();
            }
            csv_writer.flush().unwrap();
        }

        let csv_content = String::from_utf8(buffer.into_inner()).unwrap();
        let lines: Vec<&str> = csv_content.lines().collect();

        // Header + 20 data rows
        assert_eq!(lines.len(), 21, "Expected 21 lines (1 header + 20 rows)");

        // Verify 50 columns in header
        let header_cols: Vec<&str> = lines[0].split(',').collect();
        assert_eq!(header_cols.len(), 50, "Expected 50 columns");

        // Verify all rows have portfolio_manager_id = 1
        for line in &lines[1..] {
            let cols: Vec<&str> = line.split(',').collect();
            assert_eq!(cols.len(), 50, "Each row should have 50 columns");
            assert_eq!(cols[2], "1", "portfolio_manager_id should be 1");
        }
    }

    #[test]
    fn test_deterministic_output_with_seed() {
        let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let mut gen1 = DataGenerator::new(123, trade_date, 5);
        let mut gen2 = DataGenerator::new(123, trade_date, 5);

        for _ in 0..10 {
            let r1 = gen1.generate_record();
            let r2 = gen2.generate_record();
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
            seed: 42,
            trade_date: "2024-01-15".to_string(),
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
    fn test_column_names_match_schema() {
        let expected_columns = vec![
            "trade_date", "ts", "portfolio_manager_id", "fund_id", "portfolio_id",
            "account_id", "desk", "book", "strategy", "region", "country", "venue",
            "asset_class", "product", "symbol", "currency", "counterparty",
            "risk_bucket", "scenario", "trade_id", "order_id", "quantity", "price",
            "notional", "pnl", "delta", "gamma", "vega", "theta", "rho", "margin",
            "fees", "slippage", "vol", "rate", "exposure", "metric_1", "metric_2",
            "metric_3", "metric_4", "metric_5", "metric_6", "metric_7", "metric_8",
            "metric_9", "metric_10", "metric_11", "metric_12", "metric_13", "metric_14",
        ];

        let mut buffer = Cursor::new(Vec::new());
        let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut generator = DataGenerator::new(42, trade_date, 1);

        {
            let mut csv_writer = Writer::from_writer(&mut buffer);
            let record = generator.generate_record();
            csv_writer.serialize(&record).unwrap();
            csv_writer.flush().unwrap();
        }

        let csv_content = String::from_utf8(buffer.into_inner()).unwrap();
        let header = csv_content.lines().next().unwrap();
        let actual_columns: Vec<&str> = header.split(',').collect();

        assert_eq!(actual_columns, expected_columns);
    }

    #[test]
    fn test_greeks_in_valid_ranges() {
        let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut generator = DataGenerator::new(42, trade_date, 1);

        for _ in 0..100 {
            let record = generator.generate_record();
            assert!(record.delta >= -1.0 && record.delta <= 1.0);
            assert!(record.gamma >= 0.0 && record.gamma <= 0.1);
            assert!(record.theta <= 0.0);
        }
    }
}
