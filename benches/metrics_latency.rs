use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Benchmark harness for the lock-free latency tracker.
// Measures per-sample recording cost, RAII scope guard overhead,
// snapshot computation under load, and multi-threaded writer contention.
//
// The tracker uses a fixed-capacity ring buffer per phase.
// Each record() is a single atomic fetch_add(Relaxed) + relaxed store.
// Types are from the internal polyedge_engine crate (private).

fn bench_metrics_record(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_record");

    // Vary ring capacity to measure any cache-line effects.
    for &capacity in &[1024, 8192, 65536] {
        group.bench_with_input(
            BenchmarkId::new("capacity", capacity),
            &capacity,
            |b, &cap| {
                let tracker = LatencyTracker::with_capacity(cap);
                let mut i = 0u64;
                b.iter(|| {
                    i += 1;
                    tracker.record(black_box(Phase::SignalEvaluation), black_box(i * 100));
                });
            },
        );
    }
    group.finish();
}

fn bench_metrics_record_phases(c: &mut Criterion) {
    // Record to different phases in round-robin to exercise all ring buffers.
    let tracker = LatencyTracker::new();

    c.bench_function("metrics_record_round_robin", |b| {
        let phases = Phase::ALL;
        let mut i = 0u64;
        b.iter(|| {
            let phase = phases[(i as usize) % phases.len()];
            i += 1;
            tracker.record(black_box(phase), black_box(i * 100));
        });
    });
}

fn bench_metrics_scope_guard(c: &mut Criterion) {
    // Measures create + drop of the borrowing ScopeGuard.
    // The guard records elapsed nanos on drop; this captures RAII overhead
    // excluding the user's workload between create and drop.
    let tracker = LatencyTracker::new();

    c.bench_function("metrics_scope_guard", |b| {
        b.iter(|| {
            let _guard = tracker.scope(black_box(Phase::VenueSubmission));
            // Immediate drop — measures guard overhead only.
        });
    });
}

fn bench_metrics_owned_scope_guard(c: &mut Criterion) {
    // OwnedScopeGuard holds an Arc; measures Arc clone + atomic increment cost
    // relative to the borrowing guard.
    let tracker = Arc::new(LatencyTracker::new());

    c.bench_function("metrics_owned_scope_guard", |b| {
        b.iter(|| {
            let _guard = OwnedScopeGuard::new(tracker.clone(), black_box(Phase::VenueSubmission));
        });
    });
}

fn bench_metrics_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_snapshot");

    // Snapshot cost scales with ring capacity (copy + sort).
    for &fill_count in &[100, 1000, 8192] {
        group.bench_with_input(
            BenchmarkId::new("samples", fill_count),
            &fill_count,
            |b, &count| {
                let tracker = LatencyTracker::new();
                for i in 1..=count as u64 {
                    tracker.record(Phase::EndToEnd, i * 500);
                    tracker.record(Phase::SignalEvaluation, i * 200);
                }

                b.iter(|| {
                    black_box(tracker.snapshot_all());
                });
            },
        );
    }
    group.finish();
}

fn bench_metrics_contended_writers(c: &mut Criterion) {
    // Multi-threaded writers to a single phase, measuring per-record cost
    // under contention at 1, 2, and 4 concurrent writers.
    let mut group = c.benchmark_group("metrics_contended");

    for &threads in &[1, 2, 4] {
        group.bench_with_input(
            BenchmarkId::new("writers", threads),
            &threads,
            |b, &thread_count| {
                let tracker = Arc::new(LatencyTracker::new());
                b.iter_custom(|iters| {
                    let barrier = Arc::new(std::sync::Barrier::new(thread_count));
                    let mut handles = Vec::with_capacity(thread_count);
                    let start = std::time::Instant::now();

                    for _ in 0..thread_count {
                        let t = tracker.clone();
                        let bar = barrier.clone();
                        handles.push(std::thread::spawn(move || {
                            bar.wait();
                            for i in 0..iters {
                                t.record(Phase::EndToEnd, i * 100);
                            }
                        }));
                    }
                    for h in handles {
                        h.join().unwrap();
                    }
                    start.elapsed()
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_metrics_record,
    bench_metrics_record_phases,
    bench_metrics_scope_guard,
    bench_metrics_owned_scope_guard,
    bench_metrics_snapshot,
    bench_metrics_contended_writers,
);
criterion_main!(benches);
