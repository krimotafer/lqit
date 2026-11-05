use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use minigrep::matcher::Matcher;
use minigrep::parser::Config;

pub fn bench_matcher(c: &mut Criterion) {
    let text = include_str!("../data/pg100.txt");
    let file_bytes = text.len() as u64;

    // Hit
    let config_exact_hit = Config {
        pattern: "the",
        file_path: "",
        ignore_case: false,
        line_numbers: false,
    };

    let matcher_exact_hit = Matcher::from_config(&config_exact_hit);

    let config_ci_hit = Config {
        pattern: "the",
        file_path: "",
        ignore_case: true,
        line_numbers: false,
    };
    let matcher_ci_hit = Matcher::from_config(&config_ci_hit);

    // Miss
    let config_exact_miss = Config {
        pattern: "zzzzzzzz",
        file_path: "",
        ignore_case: false,
        line_numbers: false,
    };
    let matcher_exact_miss = Matcher::from_config(&config_exact_miss);

    let config_ci_miss = Config {
        pattern: "zzzzzzzz",
        file_path: "",
        ignore_case: true,
        line_numbers: false,
    };
    let matcher_ci_miss = Matcher::from_config(&config_ci_miss);

    // Count matches
    let count_exact = matcher_exact_hit.find_iter(text).count();
    let count_ci = matcher_ci_hit.find_iter(text).count();
    let count_miss = matcher_exact_miss.find_iter(text).count();

    println!("\n================== DATASET ==================");
    println!(
        "Text size              : {:.2} MB",
        file_bytes as f64 / 1_000_000.0
    );
    println!("Matches for 'the'      : {} (Exact)", count_exact);
    println!("Matches for 'the' (-i) : {} (Case Insensitive)", count_ci);
    println!("Matches for 'zzzzzzzz' : {} (Miss)", count_miss);
    println!("======================================================\n");

    // hist group
    let mut group_hit = c.benchmark_group("matcher_hit");
    group_hit.throughput(Throughput::Bytes(file_bytes));

    group_hit.bench_function("exact", |b| {
        b.iter(|| {
            for m in matcher_exact_hit.find_iter(black_box(text)) {
                black_box(m);
            }
        });
    });

    group_hit.bench_function("case_insensitive", |b| {
        b.iter(|| {
            for m in matcher_ci_hit.find_iter(black_box(text)) {
                black_box(m);
            }
        });
    });
    group_hit.finish();

    // miss group
    let mut group_miss = c.benchmark_group("matcher_miss");
    group_miss.throughput(Throughput::Bytes(file_bytes));

    group_miss.bench_function("exact", |b| {
        b.iter(|| {
            for m in matcher_exact_miss.find_iter(black_box(text)) {
                black_box(m);
            }
        });
    });

    group_miss.bench_function("case_insensitive", |b| {
        b.iter(|| {
            for m in matcher_ci_miss.find_iter(black_box(text)) {
                black_box(m);
            }
        });
    });
    group_miss.finish();
}

criterion_group!(benches, bench_matcher);
criterion_main!(benches);
