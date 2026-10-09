use criterion::{Criterion, black_box, criterion_group, criterion_main};
use minigrep::lexer::Lexer;

pub fn bench_lexer(c: &mut Criterion) {
    let flags = [
        "-cnh".to_string(),
        "pattern".to_string(),
        "--foo".to_string(),
        "file.txt".to_string(),
        "-i".to_string(),
        "--".to_string(),
    ];

    c.bench_function("lexer_mixed_args", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(&flags));
            for token in lexer {
                black_box(token);
            }
        })
    });
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
