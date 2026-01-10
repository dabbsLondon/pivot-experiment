# Plan README

## Current status

- Rust workspace with a minimal API service (`services/api`) and data generator (`tools/data-gen`).
- Web app scaffold in `apps/web` with a placeholder dev script.
- Local infrastructure in `docker-compose.yml` for ClickHouse and Redis.

## Architecture diagram

```mermaid
flowchart LR
  Web[Web app\napps/web] -->|HTTP| API[API service\nservices/api]
  API -->|Reads/Writes| ClickHouse[(ClickHouse)]
  API -->|Cache| Redis[(Redis)]
  DataGen[Data generator\ntools/data-gen] -->|Builds| Parquet[(Parquet files)]
  Parquet -->|Load| ClickHouse

  classDef web fill:#dbeafe,stroke:#1e3a8a,stroke-width:2px,color:#0f172a;
  classDef api fill:#fef3c7,stroke:#92400e,stroke-width:2px,color:#0f172a;
  classDef db fill:#bbf7d0,stroke:#166534,stroke-width:2px,color:#0f172a;
  classDef cache fill:#fde68a,stroke:#92400e,stroke-width:2px,color:#0f172a;
  classDef parquet fill:#e9d5ff,stroke:#6b21a8,stroke-width:2px,color:#0f172a;

  class Web web;
  class API api;
  class ClickHouse db;
  class Redis cache;
  class DataGen api;
  class Parquet parquet;
```

## Near-term plan

1. Implement API routes for query and pivot operations.
2. Build a basic web UI to request and visualize pivots.
3. Add integration with ClickHouse and Redis.
4. Expand test coverage with unit and integration tests.

## Data generation

The data generator (`tools/data-gen`) can create a single day of financial data with one row per PM. Configure the run with CLI flags so you can scale from a tiny test run to the long-term target of 20 million rows. Each run writes a sample CSV file (default `sample.csv`) showing the first rows used for validation.

Example: generate 10 rows for a single day.

```bash
cargo run -p pivot-data-gen -- --date 2024-01-01 --pms 10
```

Example: write a 5-row sample file to a custom path.

```bash
cargo run -p pivot-data-gen -- --date 2024-01-01 --pms 10 --sample-output docs/assets/sample.csv --sample-size 5
```

Example: generate the 20 million row target (one row per PM).

```bash
cargo run -p pivot-data-gen -- --date 2024-01-01 --pms 20000000
```

## Testing

Unit tests for the data generator live alongside the CLI in `tools/data-gen/src/main.rs`. Run them with:

```bash
cargo test -p pivot-data-gen
```
