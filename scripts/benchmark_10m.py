#!/usr/bin/env python3
"""Quick benchmark for 10M dataset"""

import requests
import time
import subprocess

API_URL = "http://localhost:8080"

def clear_cache():
    subprocess.run(["docker", "exec", "pivot-redis", "redis-cli", "FLUSHALL"], capture_output=True)

def benchmark(name, method, endpoint, data=None):
    start = time.perf_counter()
    if method == "GET":
        r = requests.get(f"{API_URL}{endpoint}")
    else:
        r = requests.post(f"{API_URL}{endpoint}", json=data)
    total = (time.perf_counter() - start) * 1000

    result = r.json()
    meta = result.get("metadata", {})
    query_ms = meta.get("query_time_ms", 0)
    cached = meta.get("cached", False)
    rows = meta.get("returned_rows", 0)

    print(f"{name:<20} | Total: {total:>7.1f}ms | Query: {query_ms:>6}ms | Rows: {rows:>4} | Cached: {cached}")
    return {"total": total, "query": query_ms, "cached": cached}

print("=" * 80)
print("10M DATASET BENCHMARK (18.6M rows after constituent explosion)")
print("=" * 80)

# Get row count
result = subprocess.run(
    ["docker", "exec", "pivot-clickhouse", "clickhouse-client", "--query", "SELECT count() FROM pivot.trades_1d"],
    capture_output=True, text=True
)
print(f"Rows in database: {int(result.stdout.strip()):,}")
print()

# Clear cache
clear_cache()
print("Cache cleared")
print()

print("--- PIVOT QUERIES (1-5 dimensions) ---")
dims_list = [
    (["asset_class"], "1 dimension"),
    (["asset_class", "region"], "2 dimensions"),
    (["asset_class", "region", "desk"], "3 dimensions"),
    (["asset_class", "region", "desk", "book"], "4 dimensions"),
    (["asset_class", "region", "desk", "book", "symbol"], "5 dimensions"),
]

pivot_results = []
for dims, name in dims_list:
    r = benchmark(name, "POST", "/api/v1/pivot", {
        "dimensions": dims,
        "metrics": ["notional", "pnl"],
        "filters": {"trade_date": "2024-01-15"}
    })
    pivot_results.append((name, r))

print()
print("--- OTHER ENDPOINTS ---")
benchmark("exposure", "GET", "/api/v1/exposure?trade_date=2024-01-15&group_by=asset_class")
benchmark("pnl", "GET", "/api/v1/pnl?trade_date=2024-01-15&group_by=portfolio_manager_id")
benchmark("instruments", "GET", "/api/v1/instruments")
benchmark("constituents", "GET", "/api/v1/constituents")

print()
print("--- REDIS CACHE TEST ---")
clear_cache()

# Cache miss
miss = benchmark("cache_miss", "POST", "/api/v1/pivot", {
    "dimensions": ["asset_class", "region"],
    "metrics": ["notional", "pnl"],
    "filters": {"trade_date": "2024-01-15"}
})

# Cache hit
hit = benchmark("cache_hit", "POST", "/api/v1/pivot", {
    "dimensions": ["asset_class", "region"],
    "metrics": ["notional", "pnl"],
    "filters": {"trade_date": "2024-01-15"}
})

speedup = miss["total"] / hit["total"] if hit["total"] > 0 else 0
print(f"\nCache speedup: {speedup:.1f}x")

print()
print("=" * 80)
print("SUMMARY")
print("=" * 80)
print(f"{'Dimensions':<15} | {'Query Time':>12}")
print("-" * 30)
for name, r in pivot_results:
    print(f"{name:<15} | {r['query']:>10}ms")
