use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Clone)]
struct GenerationConfig {
    date: String,
    num_pms: usize,
    sample_output: String,
    sample_size: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct FinancialRow {
    pm_id: u32,
    date: String,
    pnl: f64,
    aum: f64,
}

impl GenerationConfig {
    fn new(date: String, num_pms: usize, sample_output: String, sample_size: usize) -> Self {
        Self {
            date,
            num_pms,
            sample_output,
            sample_size,
        }
    }
}

fn generate_day_data(config: &GenerationConfig) -> Vec<FinancialRow> {
    let mut rows = Vec::with_capacity(config.num_pms);
    for pm_index in 0..config.num_pms {
        let pm_id = pm_index as u32 + 1;
        let pnl = calculate_pnl(pm_id);
        let aum = calculate_aum(pm_id);

        rows.push(FinancialRow {
            pm_id,
            date: config.date.clone(),
            pnl,
            aum,
        });
    }

    rows
}

fn calculate_pnl(pm_id: u32) -> f64 {
    let seed = pm_id as f64 * 1.37;
    (seed.sin() * 10_000.0).round() / 100.0
}

fn calculate_aum(pm_id: u32) -> f64 {
    let base = 100_000_000.0;
    base + (pm_id as f64 * 25_000.0)
}

fn parse_args() -> GenerationConfig {
    let mut date = String::from("2024-01-01");
    let mut num_pms = 1usize;
    let mut sample_output = String::from("sample.csv");
    let mut sample_size = 10usize;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--date" => {
                if let Some(value) = args.next() {
                    date = value;
                }
            }
            "--pms" => {
                if let Some(value) = args.next() {
                    if let Ok(parsed) = value.parse::<usize>() {
                        num_pms = parsed;
                    }
                }
            }
            "--sample-output" => {
                if let Some(value) = args.next() {
                    sample_output = value;
                }
            }
            "--sample-size" => {
                if let Some(value) = args.next() {
                    if let Ok(parsed) = value.parse::<usize>() {
                        sample_size = parsed;
                    }
                }
            }
            _ => {}
        }
    }

    GenerationConfig::new(date, num_pms, sample_output, sample_size)
}

fn main() {
    let config = parse_args();
    let rows = generate_day_data(&config);
    if let Err(error) = write_sample_csv(&rows, &config.sample_output, config.sample_size) {
        eprintln!("Failed to write sample file: {error}");
    }

    println!(
        "Generated {} rows for {} PMs on {}",
        rows.len(),
        config.num_pms,
        config.date
    );
    println!(
        "Sample output ({} rows) written to {}",
        rows.len().min(config.sample_size),
        config.sample_output
    );
}

fn write_sample_csv<P: AsRef<Path>>(
    rows: &[FinancialRow],
    path: P,
    max_rows: usize,
) -> io::Result<usize> {
    let mut file = File::create(path)?;
    writeln!(file, "pm_id,date,pnl,aum")?;
    let count = rows.len().min(max_rows);

    for row in rows.iter().take(count) {
        writeln!(file, "{},{},{},{}", row.pm_id, row.date, row.pnl, row.aum)?;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_expected_row_count_for_day() {
        let config = GenerationConfig::new(
            "2024-01-01".to_string(),
            10,
            "sample.csv".to_string(),
            10,
        );

        let rows = generate_day_data(&config);

        assert_eq!(rows.len(), 10);
        assert_eq!(rows.first().unwrap().pm_id, 1);
        assert_eq!(rows.last().unwrap().pm_id, 10);
        assert!(rows.iter().all(|row| row.date == "2024-01-01"));
    }

    #[test]
    fn writes_sample_csv_with_header_and_rows() {
        let config = GenerationConfig::new(
            "2024-01-01".to_string(),
            2,
            "sample.csv".to_string(),
            2,
        );
        let rows = generate_day_data(&config);
        let path = env::temp_dir().join("pivot_sample.csv");

        let written = write_sample_csv(&rows, &path, 2).expect("sample write failed");

        assert_eq!(written, 2);
        let contents = std::fs::read_to_string(&path).expect("sample read failed");
        let mut lines = contents.lines();
        assert_eq!(lines.next(), Some("pm_id,date,pnl,aum"));
        assert!(lines.next().is_some());
        assert!(lines.next().is_some());
    }
}
