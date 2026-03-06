# PolyEdge Engine — Benchmarks

Public benchmark reports for the PolyEdge execution engine: a high-performance
order execution and market signal system built in Rust.

The engine source is proprietary. This repository contains benchmark harnesses
and Criterion-generated reports only.

---

## Results at a Glance

### Signal & Execution Path

| Benchmark | Median | Notes |
|---|---|---|
| `engine_tick` window=4 | **273 ns** | Single-timeframe baseline |
| `engine_tick` window=8 | **353 ns** | |
| `engine_tick` window=16 | **419 ns** | |
| `engine_tick_multiframe` depth=0 | **395 ns** | No filter overhead |
| `engine_tick_multiframe` depth=4 | **417 ns** | +6% over baseline |
| `engine_tick_multiframe` depth=6 | **432 ns** | +9% over baseline |
| `intent_pipeline` | **2.32 µs** | Tick → signal → L2 check → intent |
| `opportunity_evaluator` | **1.65 µs** | Pure scoring, no I/O |
| `pair_calculator` 1 pair | **409 ns** | |
| `pair_calculator` 5 pairs | **1.25 µs** | |
| `pair_calculator` 10 pairs | **2.28 µs** | Near-linear scaling |

### Event Journal

| Benchmark | Median | Notes |
|---|---|---|
| `event_log_write` ×1 | **4.0 µs** | −67% vs prior run |
| `event_log_write` ×10 | **19.1 µs** | 1.9 µs/event — −53% vs prior run |
| `event_log_write` ×100 | **167 µs** | 1.7 µs/event — −59% vs prior run |
| `event_log_write` ×500 | **820 µs** | 1.6 µs/event — −35% vs prior run |
| `event_log_write_durable` ×1 | **4.60 ms** | fsync per write; worst-case recovery path |
| `event_log_write_durable` ×10 | **46.8 ms** | 4.68 ms/event |
| `event_log_write_durable` ×50 | **228 ms** | 4.56 ms/event |
| `event_log_replay` ×100 | **13.9 ms** @ 7.2 K elem/s | −18% vs prior run |
| `event_log_replay` ×500 | **15.6 ms** @ 32.0 K elem/s | |
| `event_log_replay` ×1000 | **14.7 ms** @ 67.8 K elem/s | −15% vs prior run |
| `serialize_event` | **334 ns** | Bincode |
| `deserialize_event` | **224 ns** | Bincode |

---

## Benchmark Environment

| | |
|---|---|
| **CPU** | Apple M4, MacBook Pro |
| **RAM** | 16 GB unified memory |
| **OS** | macOS Sequoia 15.5 |
| **Rust** | 1.92.0 (ded5c06cf 2025-12-08) |
| **Criterion** | 0.5.1 |

Benchmarks were run on Apple Silicon. Production target is Linux/x86_64;
figures are for relative component comparison, not absolute latency guarantees.

---

## Interactive Reports

Full Criterion HTML reports (PDF of slope, regression plots, MAD, SD) are in [`criterion/`](criterion/).

| Benchmark | Report |
|---|---|
| `engine_tick` | [summary](criterion/engine_tick/report/index.html) · [w=4](criterion/engine_tick/window/4/report/index.html) · [w=8](criterion/engine_tick/window/8/report/index.html) · [w=16](criterion/engine_tick/window/16/report/index.html) |
| `engine_tick_multiframe` | [summary](criterion/engine_tick_multiframe/report/index.html) · [d=0](criterion/engine_tick_multiframe/filter_depth/0/report/index.html) · [d=4](criterion/engine_tick_multiframe/filter_depth/4/report/index.html) · [d=6](criterion/engine_tick_multiframe/filter_depth/6/report/index.html) |
| `intent_pipeline` | [report](criterion/intent_pipeline/report/index.html) |
| `opportunity_evaluator` | [report](criterion/opportunity_evaluator/report/index.html) |
| `pair_calculator` | [summary](criterion/pair_calculator/report/index.html) · [1 pair](criterion/pair_calculator/markets/1/report/index.html) · [5 pairs](criterion/pair_calculator/markets/5/report/index.html) · [10 pairs](criterion/pair_calculator/markets/10/report/index.html) |
| `event_log_write` | [summary](criterion/event_log_write/report/index.html) · [×1](criterion/event_log_write/1/report/index.html) · [×10](criterion/event_log_write/10/report/index.html) · [×100](criterion/event_log_write/100/report/index.html) · [×500](criterion/event_log_write/500/report/index.html) |
| `event_log_write_durable` | [summary](criterion/event_log_write_durable/report/index.html) · [×1](criterion/event_log_write_durable/1/report/index.html) · [×10](criterion/event_log_write_durable/10/report/index.html) · [×50](criterion/event_log_write_durable/50/report/index.html) |
| `event_log_replay` | [summary](criterion/event_log_replay/report/index.html) · [×100](criterion/event_log_replay/100/report/index.html) · [×500](criterion/event_log_replay/500/report/index.html) · [×1000](criterion/event_log_replay/1000/report/index.html) |
| `event_log_serialization` | [summary](criterion/event_log_serialization/report/index.html) · [serialize](criterion/event_log_serialization/serialize_event/report/index.html) · [deserialize](criterion/event_log_serialization/deserialize_event/report/index.html) |

---

## Benchmark Harnesses

Source is in [`benches/`](benches/). Internal engine types are from the private
`polyedge-engine` crate and are not included; the harnesses are provided to show
benchmark structure and design intent.

| File | Covers |
|---|---|
| [`signal_latency.rs`](benches/signal_latency.rs) | `engine_tick`, `engine_tick_multiframe`, `intent_pipeline`, `pair_calculator`, `opportunity_evaluator` |
| [`journal_throughput.rs`](benches/journal_throughput.rs) | `event_log_write`, `event_log_write_durable`, `event_log_replay`, `event_log_serialization` |
