use std::env;
use std::io::{BufWriter, ErrorKind, stdout};
use std::process;

use minigrep::lexer::Lexer;
use minigrep::parser::{Config, ParserError};
use minigrep::searcher::Searcher;

fn main() {
    let user_args: Vec<String> = env::args().collect();

    // we skip the first element that represent the name of the binary
    let lexer = Lexer::new(user_args.get(1..).unwrap_or(&[]));

    let config = Config::parse(lexer).unwrap_or_else(|err| match err {
        ParserError::HelpRequest => {
            println!("{err}");
            process::exit(0);
        }
        _ => {
            eprint!("minigrep: {err}");
            process::exit(1);
        }
    });

    let searcher = Searcher::from_config(&config);

    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());

    if let Err(err) = searcher.search_file(config.file_path, &mut writer) {
        if err.kind() == ErrorKind::BrokenPipe {
            process::exit(0);
        }
        eprintln!("minigrep: {}: {err}", config.file_path);
        process::exit(1);
    }
}
