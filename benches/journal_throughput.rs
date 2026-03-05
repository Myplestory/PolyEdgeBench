use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// Benchmark harness for the append-only event log (execution journal).
// Measures write throughput, durable write latency, replay speed,
// and serialization cost at varying batch sizes.
//
// Types are from the internal polyedge_engine crate (private).

fn bench_event_log_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_log_write");

    for batch_size in [1, 10, 100, 500] {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &size| {
                b.iter_custom(|iters| {
                    let mut total = std::time::Duration::ZERO;
                    for _ in 0..iters {
                        let log = EventLog::new_ephemeral();
                        let start = std::time::Instant::now();
                        for _ in 0..size {
                            let _ = log.append(black_box(make_event()));
                        }
                        total += start.elapsed();
                        drop(log);
                    }
                    total
                });
            },
        );
    }
    group.finish();
}

fn bench_event_log_write_durable(c: &mut Criterion) {
    // Durable writes fsync before acking; measures worst-case latency
    // for the recovery guarantee path.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("event_log_write_durable");

    for batch_size in [1, 10, 50] {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &size| {
                b.iter_custom(|iters| {
                    let mut total = std::time::Duration::ZERO;
                    for _ in 0..iters {
                        let log = EventLog::new_ephemeral();
                        let start = std::time::Instant::now();
                        rt.block_on(async {
                            for _ in 0..size {
                                let _ = log.append_durable(black_box(make_event())).await;
                            }
                        });
                        total += start.elapsed();
                        drop(log);
                    }
                    total
                });
            },
        );
    }
    group.finish();
}

fn bench_event_log_replay(c: &mut Criterion) {
    // Replay reads and deserializes all entries from disk.
    // Used on startup for crash recovery; measures recovery time at scale.
    let mut group = c.benchmark_group("event_log_replay");

    for entry_count in [100, 500, 1000] {
        let log = EventLog::new_ephemeral();
        for _ in 0..entry_count {
            let _ = log.append(make_status_event());
        }
        drop(log.writer());

        group.throughput(Throughput::Elements(entry_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(entry_count),
            &entry_count,
            |b, _| {
                b.iter(|| {
                    let entries = EventLog::replay(log.path()).unwrap();
                    black_box(entries.len());
                });
            },
        );
    }
    group.finish();
}

fn bench_event_log_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_log_serialization");

    let event = make_event();
    let bytes = serialize(&event);

    group.bench_function("serialize_event", |b| {
        b.iter(|| {
            black_box(serialize(black_box(&event)));
        });
    });

    group.bench_function("deserialize_event", |b| {
        b.iter(|| {
            black_box(deserialize(black_box(&bytes)));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_event_log_write,
    bench_event_log_write_durable,
    bench_event_log_replay,
    bench_event_log_serialization,
);
criterion_main!(benches);
