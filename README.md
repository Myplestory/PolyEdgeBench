# PolyEdge Engine — Benchmarks

Public benchmark reports for the PolyEdge execution engine, a high-performance
order execution and market signal system built in Rust.

The engine source is proprietary. This repository contains benchmark harnesses
and Criterion-generated reports only.

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

## Reports

Full interactive Criterion reports are available in [`criterion/report/index.html`](criterion/report/index.html).

| Benchmark | Description |
|---|---|
| `engine_tick` | Per-tick latency of the signal evaluation pipeline across varying window sizes |
| `engine_tick_multiframe` | Overhead of the multi-timeframe filter at varying filter depths |
| `intent_pipeline` | Full path: tick → signal evaluation → liquidity check → intent output |
| `pair_calculator` | Cross-market pair spread calculation scaling across 1, 5, and 10 active pairs |
| `opportunity_evaluator` | Isolated opportunity scoring — pure compute, no I/O |
| `event_log_write` | Append throughput at batch sizes 1, 10, 100, 500 |
| `event_log_write_durable` | Durable (fsync) write latency for the crash recovery guarantee path |
| `event_log_replay` | Startup recovery replay speed at 100, 500, and 1000 entries |
| `event_log_serialization` | Bincode serialization and deserialization cost per event |

## Source

Benchmark harnesses (pseudocode form, internal types omitted) are in [`benches/`](benches/).
