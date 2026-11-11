use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use minigrep::parser::Config;
use minigrep::searcher::Searcher;
use std::io::{BufReader, sink};

pub fn bench_searcher(c: &mut Criterion) {
    let text = include_str!("../data/pg100.txt");
    let file_bytes = text.len() as u64;

    println!("\n================== SEARCHER BENCHMARK ==================");
    println!(
        "Corpus size            : {:.2} MB (196,399 lines)",
        file_bytes as f64 / 1_000_000.0
    );

    let mut group = c.benchmark_group("searcher");
    group.throughput(Throughput::Bytes(file_bytes));

    // 1. Realistic / Sparse Hit
    let config_realistic = Config {
        pattern: "Hamlet",
        file_path: "",
        ignore_case: false,
        line_numbers: true,
        color: true,
    };
    let searcher_realistic = Searcher::from_config(&config_realistic);

    group.bench_function("realistic_sparse_hit", |b| {
        b.iter(|| {
            // BufReader
            let mut reader = BufReader::new(black_box(text.as_bytes()));
            let mut writer = sink();
            black_box(searcher_realistic.search(&mut reader, &mut writer).unwrap());
        });
    });

    // 2. Dense Hit (Stress test)
    let config_dense = Config {
        pattern: "the",
        file_path: "",
        ignore_case: true,
        line_numbers: true,
        color: true,
    };
    let searcher_dense = Searcher::from_config(&config_dense);

    group.bench_function("dense_stress_hit", |b| {
        b.iter(|| {
            let mut reader = BufReader::new(black_box(text.as_bytes()));
            let mut writer = sink();
            black_box(searcher_dense.search(&mut reader, &mut writer).unwrap());
        });
    });

    // 3. Pure Scan / Miss
    let config_miss = Config {
        pattern: "zzzzzzzz",
        file_path: "",
        ignore_case: false,
        line_numbers: false,
        color: false,
    };
    let searcher_miss = Searcher::from_config(&config_miss);

    group.bench_function("pure_miss_scan", |b| {
        b.iter(|| {
            let mut reader = BufReader::new(black_box(text.as_bytes()));
            let mut writer = sink();
            black_box(searcher_miss.search(&mut reader, &mut writer).unwrap());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_searcher);
criterion_main!(benches);
