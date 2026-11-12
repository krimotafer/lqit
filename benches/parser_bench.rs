use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lqit::lexer::Lexer;
use lqit::parser::Config;

pub fn bench_parser(c: &mut Criterion) {
    let flags = [
        "-in".to_string(),
        "pattern".to_string(),
        "file.txt".to_string(),
    ];

    c.bench_function("parser_check", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(&flags));
            let config = Config::parse(black_box(lexer)).unwrap();
            black_box(config.pattern);
            black_box(config.file_path);
            black_box(config.ignore_case);
            black_box(config.line_numbers);
        })
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
