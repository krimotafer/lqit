use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use minigrep::matcher::{Match, Matcher};
use minigrep::parser::Config;
use minigrep::printer::Printer;

pub fn bench_printer(c: &mut Criterion) {
    let text = include_str!("../data/pg100.txt");

    let config_matcher = Config {
        pattern: "the",
        file_path: "",
        ignore_case: false,
        line_numbers: false,
        color: false,
    };
    let matcher = Matcher::from_config(&config_matcher);

    let mut lines_data: Vec<(&str, Vec<Match>, usize)> = Vec::new();
    let mut total_bytes: u64 = 0;

    for (idx, line) in text.lines().enumerate() {
        let matches: Vec<Match> = matcher.find_iter(line).collect();
        if !matches.is_empty() {
            total_bytes += line.len() as u64;
            lines_data.push((line, matches, idx + 1));
        }
    }

    println!("\n================== PRINTER DATASET ==================");
    println!("Matching lines to print: {}", lines_data.len());
    println!(
        "Total payload size     : {:.2} MB",
        total_bytes as f64 / 1_000_000.0
    );

    let printer_raw = Printer::from_config(&Config {
        pattern: "the",
        file_path: "",
        ignore_case: false,
        line_numbers: false,
        color: false,
    });

    let printer_n = Printer::from_config(&Config {
        pattern: "the",
        file_path: "",
        ignore_case: false,
        line_numbers: true,
        color: false,
    });

    let printer_c = Printer::from_config(&Config {
        pattern: "the",
        file_path: "",
        ignore_case: false,
        line_numbers: false,
        color: true,
    });

    let printer_all = Printer::from_config(&Config {
        pattern: "the",
        file_path: "",
        ignore_case: false,
        line_numbers: true,
        color: true,
    });

    // WARN: we don't write into `io::sink()` anymore. The compiler can see that sink does
    // nothing, so it deletes the printer work and the bench measures only the loop.
    // We write into a Vec<u8> allocated once here, cleared at each iteration (the capacity stays,
    // so no allocation in the timed part) and we `black_box` it at the end so the output must exist.
    // Writing into a Vec is a memory copy, close to what BufWriter does in main.
    let mut writer: Vec<u8> = Vec::with_capacity(8 * 1024 * 1024);

    let mut group = c.benchmark_group("printer");
    group.throughput(Throughput::Bytes(total_bytes));

    group.bench_function("raw", |b| {
        b.iter(|| {
            writer.clear();
            for (line, matches, line_num) in &lines_data {
                printer_raw
                    .print_line(&mut writer, black_box(line), black_box(matches), *line_num)
                    .unwrap();
            }
            black_box(&writer);
        });
    });

    group.bench_function("line_numbers", |b| {
        b.iter(|| {
            writer.clear();
            for (line, matches, line_num) in &lines_data {
                printer_n
                    .print_line(&mut writer, black_box(line), black_box(matches), *line_num)
                    .unwrap();
            }
            black_box(&writer);
        });
    });

    group.bench_function("color", |b| {
        b.iter(|| {
            writer.clear();
            for (line, matches, line_num) in &lines_data {
                printer_c
                    .print_line(&mut writer, black_box(line), black_box(matches), *line_num)
                    .unwrap();
            }
            black_box(&writer);
        });
    });

    group.bench_function("all_flags", |b| {
        b.iter(|| {
            writer.clear();
            for (line, matches, line_num) in &lines_data {
                printer_all
                    .print_line(&mut writer, black_box(line), black_box(matches), *line_num)
                    .unwrap();
            }
            black_box(&writer);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_printer);
criterion_main!(benches);
