use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Benchmark harness for the core engine tick path.
// Measures per-tick latency of the signal evaluation pipeline
// across varying window sizes and multi-timeframe filter depths.
//
// Types are from the internal polyedge_engine crate (private).

fn bench_engine_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_tick");

    for &window in &[4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("window", window),
            &window,
            |b, &window| {
                // Configure engine with varying window size; pre-fill to warm state.
                let mut engine = engine_with_window(window);
                pre_fill(&mut engine, window);

                let mut idx = window as i64 + 2;
                b.iter(|| {
                    black_box(engine.on_tick(black_box(make_tick(idx))));
                    idx += 1;
                });
            },
        );
    }
    group.finish();
}

fn bench_engine_tick_multiframe(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_tick_multiframe");

    // filter_depth=0: single timeframe baseline.
    // filter_depth>0: multi-timeframe filter active, N fast bars aggregated per slow bar.
    for &filter_depth in &[0, 4, 6] {
        group.bench_with_input(
            BenchmarkId::new("filter_depth", filter_depth),
            &filter_depth,
            |b, &filter_depth| {
                let mut engine = engine_with_filter(filter_depth);
                pre_fill_n(&mut engine, 100);

                let mut idx = 101i64;
                b.iter(|| {
                    black_box(engine.on_tick(black_box(make_tick(idx))));
                    idx += 1;
                });
            },
        );
    }
    group.finish();
}

fn bench_intent_pipeline(c: &mut Criterion) {
    // Full path: tick -> signal evaluation -> L2 liquidity check -> intent output.
    c.bench_function("intent_pipeline", |b| {
        let mut pipeline = IntentPipeline::new();
        pipeline.register_market("m1");
        pre_fill_pipeline(&mut pipeline, 20);

        let l2 = make_l2_snapshot();
        let mut idx = 21i64;
        b.iter(|| {
            black_box(pipeline.on_tick(black_box(make_tick(idx)), Some(&l2)));
            idx += 1;
        });
    });
}

fn bench_pair_calculator(c: &mut Criterion) {
    let mut group = c.benchmark_group("pair_calculator");

    // Scale test: 1, 5, 10 active market pairs updating concurrently.
    for &markets in &[1, 5, 10] {
        group.bench_with_input(
            BenchmarkId::new("markets", markets),
            &markets,
            |b, &count| {
                let mut calc = PairCalculator::new();
                for i in 0..count {
                    calc.register_pair(&format!("pair_{}", i), &format!("venue_a_{}", i), &format!("venue_b_{}", i));
                    calc.on_book_update(make_book_snapshot("venue_a", &format!("venue_a_{}", i)));
                    calc.on_book_update(make_book_snapshot("venue_b", &format!("venue_b_{}", i)));
                }

                let mut idx = 0usize;
                b.iter(|| {
                    let snap = make_book_snapshot("venue_a", &format!("venue_a_{}", idx % count));
                    idx += 1;
                    black_box(calc.on_book_update(black_box(snap)));
                });
            },
        );
    }
    group.finish();
}

fn bench_opportunity_evaluator(c: &mut Criterion) {
    // Isolated evaluation of a single cross-market opportunity.
    // No I/O; measures pure scoring logic latency.
    let eval = Evaluator::new();
    let opportunity = make_opportunity();

    c.bench_function("opportunity_evaluator", |b| {
        b.iter(|| {
            black_box(eval.evaluate(black_box(&opportunity)));
        });
    });
}

criterion_group!(
    benches,
    bench_engine_tick,
    bench_engine_tick_multiframe,
    bench_intent_pipeline,
    bench_pair_calculator,
    bench_opportunity_evaluator,
);
criterion_main!(benches);
