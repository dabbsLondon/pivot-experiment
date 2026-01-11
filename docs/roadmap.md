# Project Roadmap

A detailed action plan for delivering the Pivot Experiment platform.

## Current State

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           PROJECT STATUS OVERVIEW                                │
└─────────────────────────────────────────────────────────────────────────────────┘

    Component               Status          Progress
    ─────────               ──────          ────────

    ┌─────────────────────────────────────────────────────────────────────────┐
    │ Data Generator        COMPLETE        ████████████████████ 100%        │
    │ tools/data-gen                                                          │
    │                                                                          │
    │ ✓ CLI with all options                                                  │
    │ ✓ 29 instruments (stocks, ETFs, ETCs, commodities)                     │
    │ ✓ Constituent explosion                                                 │
    │ ✓ Deterministic seeding                                                 │
    │ ✓ 8 unit tests passing                                                  │
    └─────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────────┐
    │ Database Schema       COMPLETE        ████████████████████ 100%        │
    │ sql/clickhouse                                                          │
    │                                                                          │
    │ ✓ instruments table                                                     │
    │ ✓ constituents table                                                    │
    │ ✓ trades_1d fact table (55 columns)                                    │
    │ ✓ Rollup table with materialized view                                   │
    │ ✓ Partitioning by trade_date                                           │
    └─────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────────┐
    │ Infrastructure        COMPLETE        ████████████████████ 100%        │
    │ docker-compose.yml                                                      │
    │                                                                          │
    │ ✓ ClickHouse 24.2                                                       │
    │ ✓ Redis 7.2                                                             │
    │ ✓ Volume mounts                                                         │
    │ ✓ SQL scripts mounted                                                   │
    └─────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────────┐
    │ CI/CD Pipeline        COMPLETE        ████████████████████ 100%        │
    │ .github/workflows                                                       │
    │                                                                          │
    │ ✓ Rust tests                                                            │
    │ ✓ Sample data generation                                                │
    │ ✓ Docker build verification                                             │
    │ ✓ Job summary with metrics                                              │
    └─────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────────┐
    │ API Server            SCAFFOLD        ████░░░░░░░░░░░░░░░░  5%         │
    │ services/api                                                            │
    │                                                                          │
    │ ✓ Project structure                                                     │
    │ ○ HTTP framework (Actix/Axum)                                          │
    │ ○ ClickHouse client                                                     │
    │ ○ Query endpoints                                                       │
    │ ○ Redis caching                                                         │
    └─────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────────┐
    │ Web Application       SCAFFOLD        ████░░░░░░░░░░░░░░░░  5%         │
    │ apps/web                                                                │
    │                                                                          │
    │ ✓ Package structure                                                     │
    │ ○ React + TypeScript setup                                              │
    │ ○ Pivot table component                                                 │
    │ ○ API integration                                                       │
    │ ○ Styling                                                               │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## Delivery Phases

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              PHASE OVERVIEW                                      │
└─────────────────────────────────────────────────────────────────────────────────┘

 PHASE 1              PHASE 2              PHASE 3              PHASE 4
 ───────              ───────              ───────              ───────
 Data Layer           API Server           Web Frontend         Production
 (COMPLETE)           (IN PROGRESS)        (PLANNED)            (PLANNED)

┌──────────┐       ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ ✓ Schema │       │ HTTP Server  │     │ React App    │     │ Performance  │
│ ✓ Gen    │──────▶│ ClickHouse   │────▶│ Pivot Table  │────▶│ Monitoring   │
│ ✓ CI/CD  │       │ Redis Cache  │     │ Charts       │     │ Deployment   │
└──────────┘       └──────────────┘     └──────────────┘     └──────────────┘
```

---

## Phase 1: Data Layer (COMPLETE)

All items in this phase have been delivered.

| Item | Status | Description |
|------|--------|-------------|
| ClickHouse Schema | ✓ | 3 tables + 1 materialized view |
| Data Generator | ✓ | Rust CLI with constituent explosion |
| Reference Data | ✓ | 29 instruments, 24 constituent mappings |
| CI Pipeline | ✓ | Automated testing and validation |
| Docker Setup | ✓ | ClickHouse + Redis containers |

---

## Phase 2: API Server

Build the Rust REST API that queries ClickHouse and serves pivot data.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            API SERVER ARCHITECTURE                               │
└─────────────────────────────────────────────────────────────────────────────────┘

                              ┌─────────────────────────────────────┐
                              │           API Server                 │
                              │         services/api                 │
                              └─────────────────────────────────────┘
                                              │
           ┌──────────────────────────────────┼──────────────────────────────────┐
           │                                  │                                  │
           ▼                                  ▼                                  ▼
  ┌─────────────────┐              ┌─────────────────┐              ┌─────────────────┐
  │   HTTP Layer    │              │   Service       │              │   Data Layer    │
  │                 │              │   Layer         │              │                 │
  │  Axum Router    │──requests──▶│  Query Builder  │──queries───▶│  ClickHouse     │
  │  Middleware     │             │  Aggregation    │              │  Client         │
  │  Error Handler  │◀─responses─│  Transformation │◀─results────│                 │
  └─────────────────┘              └─────────────────┘              └─────────────────┘
           │                                  │                              │
           │                                  ▼                              │
           │                       ┌─────────────────┐                       │
           │                       │   Cache Layer   │                       │
           │                       │                 │                       │
           └──────────────────────▶│     Redis       │◀──────────────────────┘
                                   │   (optional)    │
                                   └─────────────────┘
```

### Tasks

#### 2.1 Project Setup

| Task | Description | Dependencies |
|------|-------------|--------------|
| 2.1.1 | Add Axum framework dependency | - |
| 2.1.2 | Add ClickHouse client (clickhouse-rs) | - |
| 2.1.3 | Add Redis client (redis-rs) | - |
| 2.1.4 | Add Tokio runtime | - |
| 2.1.5 | Add Serde for JSON serialization | - |
| 2.1.6 | Configure tracing/logging | 2.1.4 |

**Cargo.toml additions:**

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
clickhouse = "0.11"
redis = { version = "0.24", features = ["tokio-comp"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
tower-http = { version = "0.5", features = ["cors", "trace"] }
```

#### 2.2 Database Client

| Task | Description | Dependencies |
|------|-------------|--------------|
| 2.2.1 | Create ClickHouse connection pool | 2.1.2 |
| 2.2.2 | Implement query execution wrapper | 2.2.1 |
| 2.2.3 | Add connection health check | 2.2.1 |
| 2.2.4 | Handle connection errors gracefully | 2.2.2 |

#### 2.3 API Endpoints

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              API ENDPOINTS                                       │
└─────────────────────────────────────────────────────────────────────────────────┘

    Endpoint                    Method    Description
    ────────                    ──────    ───────────

    /health                     GET       Health check (DB connectivity)
    /api/v1/pivot               POST      Execute pivot query
    /api/v1/instruments         GET       List all instruments
    /api/v1/constituents        GET       Get constituent mappings
    /api/v1/exposure            GET       Total exposure by dimension
    /api/v1/pnl                 GET       P&L aggregation
```

| Task | Description | Dependencies |
|------|-------------|--------------|
| 2.3.1 | Implement `/health` endpoint | 2.2.3 |
| 2.3.2 | Define pivot query request/response types | 2.1.5 |
| 2.3.3 | Implement `/api/v1/pivot` endpoint | 2.3.2, 2.2.2 |
| 2.3.4 | Implement `/api/v1/instruments` endpoint | 2.2.2 |
| 2.3.5 | Implement `/api/v1/constituents` endpoint | 2.2.2 |
| 2.3.6 | Implement `/api/v1/exposure` endpoint | 2.2.2 |
| 2.3.7 | Implement `/api/v1/pnl` endpoint | 2.2.2 |

**Pivot Query Request Schema:**

```json
{
  "dimensions": ["portfolio_manager_id", "asset_class", "symbol"],
  "metrics": ["notional", "pnl", "quantity"],
  "filters": {
    "trade_date": "2024-01-15",
    "exposure_type": ["Direct", "ETF", "ETC"]
  },
  "sort": { "field": "pnl", "direction": "desc" },
  "limit": 100
}
```

#### 2.4 Query Builder

| Task | Description | Dependencies |
|------|-------------|--------------|
| 2.4.1 | Create query builder struct | - |
| 2.4.2 | Implement dimension grouping | 2.4.1 |
| 2.4.3 | Implement metric aggregation | 2.4.1 |
| 2.4.4 | Implement filter conditions | 2.4.1 |
| 2.4.5 | Add SQL injection prevention | 2.4.1 |
| 2.4.6 | Support exposure_type filtering | 2.4.4 |

#### 2.5 Caching Layer

| Task | Description | Dependencies |
|------|-------------|--------------|
| 2.5.1 | Create Redis connection pool | 2.1.3 |
| 2.5.2 | Implement cache key generation | 2.5.1 |
| 2.5.3 | Add cache get/set helpers | 2.5.1 |
| 2.5.4 | Configure TTL for cached queries | 2.5.3 |
| 2.5.5 | Add cache bypass option | 2.5.3 |

#### 2.6 Middleware & Error Handling

| Task | Description | Dependencies |
|------|-------------|--------------|
| 2.6.1 | Add CORS middleware | 2.1.1 |
| 2.6.2 | Add request logging middleware | 2.1.6 |
| 2.6.3 | Implement error response types | 2.1.5 |
| 2.6.4 | Add request timeout middleware | 2.1.1 |

#### 2.7 Testing

| Task | Description | Dependencies |
|------|-------------|--------------|
| 2.7.1 | Add integration test setup | 2.3.1 |
| 2.7.2 | Write health endpoint tests | 2.3.1 |
| 2.7.3 | Write pivot query tests | 2.3.3 |
| 2.7.4 | Add query builder unit tests | 2.4.6 |

---

## Phase 3: Web Frontend

Build the React application for interactive pivot table exploration.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          WEB APPLICATION ARCHITECTURE                            │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│                               apps/web                                           │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                           Pages / Routes                                 │    │
│  │                                                                          │    │
│  │   /                     /pivot                    /settings              │    │
│  │   Dashboard            Pivot Table               User Prefs              │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                      │                                           │
│  ┌───────────────────────────────────┼───────────────────────────────────────┐  │
│  │                           Components                                       │  │
│  │                                   │                                        │  │
│  │  ┌─────────────┐  ┌──────────────┴──────────────┐  ┌─────────────────┐   │  │
│  │  │ DimensionPicker │ │      PivotTable          │  │  ExposureToggle │   │  │
│  │  │             │  │                              │  │                 │   │  │
│  │  │ Select rows │  │  ┌────┬────┬────┬────┬────┐ │  │ Direct  ✓      │   │  │
│  │  │ & columns   │  │  │    │    │    │    │    │ │  │ ETF     ✓      │   │  │
│  │  │             │  │  ├────┼────┼────┼────┼────┤ │  │ ETC     ✓      │   │  │
│  │  └─────────────┘  │  │    │    │    │    │    │ │  │ Constituent ○  │   │  │
│  │                   │  ├────┼────┼────┼────┼────┤ │  │                 │   │  │
│  │  ┌─────────────┐  │  │    │    │    │    │    │ │  └─────────────────┘   │  │
│  │  │ MetricPicker│  │  └────┴────┴────┴────┴────┘ │                        │  │
│  │  │             │  │                              │  ┌─────────────────┐   │  │
│  │  │ ○ Notional  │  │  Expandable rows            │  │  DatePicker     │   │  │
│  │  │ ✓ PnL       │  │  Sortable columns           │  │                 │   │  │
│  │  │ ○ Quantity  │  │  Click to drill-down        │  │  2024-01-15     │   │  │
│  │  └─────────────┘  └──────────────────────────────┘  └─────────────────┘   │  │
│  │                                                                            │  │
│  └────────────────────────────────────────────────────────────────────────────┘  │
│                                      │                                           │
│  ┌───────────────────────────────────┼───────────────────────────────────────┐  │
│  │                         State Management                                   │  │
│  │                                   │                                        │  │
│  │   ┌───────────────────────────────┼─────────────────────────────────┐     │  │
│  │   │                 React Query / TanStack Query                     │     │  │
│  │   │                                                                  │     │  │
│  │   │  useQuery('pivot', { dimensions, metrics, filters })            │     │  │
│  │   │  useQuery('instruments')                                        │     │  │
│  │   │  useMutation('updateFilters')                                   │     │  │
│  │   └──────────────────────────────────────────────────────────────────┘     │  │
│  │                                   │                                        │  │
│  └───────────────────────────────────┼────────────────────────────────────────┘  │
│                                      │                                           │
│  ┌───────────────────────────────────▼───────────────────────────────────────┐  │
│  │                           API Client                                       │  │
│  │                                                                            │  │
│  │   fetch('/api/v1/pivot', { method: 'POST', body: query })                 │  │
│  │                                                                            │  │
│  └────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
                              ┌─────────────────┐
                              │   API Server    │
                              │  services/api   │
                              └─────────────────┘
```

### Tasks

#### 3.1 Project Setup

| Task | Description | Dependencies |
|------|-------------|--------------|
| 3.1.1 | Initialize Vite + React + TypeScript | - |
| 3.1.2 | Configure TailwindCSS | 3.1.1 |
| 3.1.3 | Add TanStack Query | 3.1.1 |
| 3.1.4 | Configure ESLint + Prettier | 3.1.1 |
| 3.1.5 | Add React Router | 3.1.1 |
| 3.1.6 | Configure proxy to API server | 3.1.1 |

**Commands:**

```bash
cd apps/web
pnpm create vite . --template react-ts
pnpm add @tanstack/react-query react-router-dom
pnpm add -D tailwindcss postcss autoprefixer
pnpm add -D eslint prettier eslint-config-prettier
```

#### 3.2 API Client Layer

| Task | Description | Dependencies |
|------|-------------|--------------|
| 3.2.1 | Create API client module | 3.1.1 |
| 3.2.2 | Define TypeScript types for API responses | 3.2.1 |
| 3.2.3 | Create pivot query hook | 3.2.2, 3.1.3 |
| 3.2.4 | Create instruments query hook | 3.2.2, 3.1.3 |
| 3.2.5 | Add error handling wrapper | 3.2.1 |

#### 3.3 Core Components

| Task | Description | Dependencies |
|------|-------------|--------------|
| 3.3.1 | Build PivotTable component | 3.2.3 |
| 3.3.2 | Add column sorting | 3.3.1 |
| 3.3.3 | Add row expansion (drill-down) | 3.3.1 |
| 3.3.4 | Build DimensionPicker component | 3.2.4 |
| 3.3.5 | Build MetricPicker component | 3.2.4 |
| 3.3.6 | Build ExposureToggle component | - |
| 3.3.7 | Build DatePicker component | - |

#### 3.4 Layout & Navigation

| Task | Description | Dependencies |
|------|-------------|--------------|
| 3.4.1 | Create app layout shell | 3.1.5 |
| 3.4.2 | Add sidebar navigation | 3.4.1 |
| 3.4.3 | Create Dashboard page | 3.4.1 |
| 3.4.4 | Create Pivot page | 3.4.1, 3.3.1 |
| 3.4.5 | Add loading states | 3.1.3 |
| 3.4.6 | Add error boundaries | 3.4.1 |

#### 3.5 Visualization

| Task | Description | Dependencies |
|------|-------------|--------------|
| 3.5.1 | Add chart library (Recharts) | 3.1.1 |
| 3.5.2 | Build P&L bar chart | 3.5.1 |
| 3.5.3 | Build exposure pie chart | 3.5.1 |
| 3.5.4 | Add chart/table view toggle | 3.5.2, 3.3.1 |

#### 3.6 Polish & UX

| Task | Description | Dependencies |
|------|-------------|--------------|
| 3.6.1 | Add keyboard shortcuts | 3.3.1 |
| 3.6.2 | Add export to CSV | 3.3.1 |
| 3.6.3 | Add saved queries | 3.4.4 |
| 3.6.4 | Responsive design | 3.4.1 |
| 3.6.5 | Dark mode support | 3.1.2 |

---

## Phase 4: Production Readiness

Prepare for production deployment.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         PRODUCTION ARCHITECTURE                                  │
└─────────────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────────────────────┐
                    │            Load Balancer            │
                    │           (nginx/cloudflare)        │
                    └──────────────────┬──────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
              ▼                        ▼                        ▼
    ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
    │   Web (CDN)     │      │   API Pod 1     │      │   API Pod 2     │
    │   Static Files  │      │                 │      │                 │
    └─────────────────┘      └────────┬────────┘      └────────┬────────┘
                                      │                        │
                                      └───────────┬────────────┘
                                                  │
                    ┌─────────────────────────────┼─────────────────────────────┐
                    │                             │                             │
                    ▼                             ▼                             ▼
          ┌─────────────────┐           ┌─────────────────┐           ┌─────────────────┐
          │   ClickHouse    │           │   ClickHouse    │           │     Redis       │
          │   Primary       │◀─────────▶│   Replica       │           │    Cluster      │
          └─────────────────┘           └─────────────────┘           └─────────────────┘
```

### Tasks

#### 4.1 Containerization

| Task | Description | Dependencies |
|------|-------------|--------------|
| 4.1.1 | Create API Dockerfile | Phase 2 |
| 4.1.2 | Create Web Dockerfile | Phase 3 |
| 4.1.3 | Multi-stage builds for optimization | 4.1.1, 4.1.2 |
| 4.1.4 | Update docker-compose for prod | 4.1.3 |

#### 4.2 Configuration

| Task | Description | Dependencies |
|------|-------------|--------------|
| 4.2.1 | Environment variable management | 4.1.1 |
| 4.2.2 | Secrets management | 4.2.1 |
| 4.2.3 | Configuration validation | 4.2.1 |

#### 4.3 Observability

| Task | Description | Dependencies |
|------|-------------|--------------|
| 4.3.1 | Add metrics endpoint (Prometheus) | Phase 2 |
| 4.3.2 | Add distributed tracing | Phase 2 |
| 4.3.3 | Configure logging aggregation | 4.2.1 |
| 4.3.4 | Create Grafana dashboards | 4.3.1 |

#### 4.4 Performance

| Task | Description | Dependencies |
|------|-------------|--------------|
| 4.4.1 | Query performance benchmarks | Phase 2 |
| 4.4.2 | Add query caching strategy | 4.4.1 |
| 4.4.3 | Optimize ClickHouse indexes | 4.4.1 |
| 4.4.4 | Load testing (20M rows) | 4.4.1 |

#### 4.5 Deployment

| Task | Description | Dependencies |
|------|-------------|--------------|
| 4.5.1 | Create Kubernetes manifests | 4.1.4 |
| 4.5.2 | Configure CI/CD for deploy | 4.5.1 |
| 4.5.3 | Set up staging environment | 4.5.2 |
| 4.5.4 | Production deployment runbook | 4.5.3 |

---

## Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          TASK DEPENDENCY GRAPH                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

                              ┌─────────────────┐
                              │   Phase 1       │
                              │  (COMPLETE)     │
                              │                 │
                              │  ✓ Schema       │
                              │  ✓ Data Gen     │
                              │  ✓ CI/CD        │
                              └────────┬────────┘
                                       │
              ┌────────────────────────┴────────────────────────┐
              │                                                 │
              ▼                                                 ▼
    ┌─────────────────┐                               ┌─────────────────┐
    │   Phase 2       │                               │   Phase 3       │
    │   API Server    │                               │   Web Frontend  │
    │                 │                               │                 │
    │ 2.1 Setup       │                               │ 3.1 Setup ◄─────┤
    │      │          │                               │      │          │
    │      ▼          │                               │      ▼          │
    │ 2.2 DB Client   │                               │ 3.2 API Client  │
    │      │          │                               │      │          │
    │      ▼          │                               │      ▼          │
    │ 2.3 Endpoints ──┼───────────────────────────────┼─▶ 3.3 Components│
    │      │          │        API required           │      │          │
    │      ▼          │                               │      ▼          │
    │ 2.4 Query Build │                               │ 3.4 Layout      │
    │      │          │                               │      │          │
    │      ▼          │                               │      ▼          │
    │ 2.5 Cache       │                               │ 3.5 Charts      │
    │      │          │                               │      │          │
    │      ▼          │                               │      ▼          │
    │ 2.6 Middleware  │                               │ 3.6 Polish      │
    │      │          │                               │                 │
    │      ▼          │                               │                 │
    │ 2.7 Tests       │                               │                 │
    └────────┬────────┘                               └────────┬────────┘
             │                                                 │
             └────────────────────────┬────────────────────────┘
                                      │
                                      ▼
                            ┌─────────────────┐
                            │   Phase 4       │
                            │   Production    │
                            │                 │
                            │ 4.1 Containers  │
                            │      │          │
                            │      ▼          │
                            │ 4.2 Config      │
                            │      │          │
                            │      ▼          │
                            │ 4.3 Observability│
                            │      │          │
                            │      ▼          │
                            │ 4.4 Performance │
                            │      │          │
                            │      ▼          │
                            │ 4.5 Deploy      │
                            └─────────────────┘
```

---

## Quick Reference: Next Steps

To continue development, start with **Phase 2: API Server**.

**Immediate next tasks:**

1. Add dependencies to `services/api/Cargo.toml`
2. Implement ClickHouse connection
3. Create `/health` endpoint
4. Implement `/api/v1/pivot` endpoint

```bash
# Start developing the API
cd services/api
cargo add axum tokio clickhouse serde serde_json
cargo add tracing tracing-subscriber
cargo add tower-http --features cors,trace
```

See [Data Generator Guide](./getting-started.md) for the complete data setup workflow.
