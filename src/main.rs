use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

use minigrep::lexer::Lexer;
use minigrep::parser::{Config, ParserError};

fn main() {
    let user_args: Vec<String> = env::args().collect();
    let test = user_args.clone();
    for t in test {
        println!("{}", t);
    }
    // we skip the first element that represent the name of the binary
    let lexer = Lexer::new(user_args.get(1..).unwrap_or(&[]));
    let _config = Config::parse(lexer).unwrap_or_else(|err| match err {
        ParserError::HelpRequest => {
            println!("{err}");
            process::exit(0);
        }
        _ => {
            eprint!("minigrep: {err}");
            process::exit(1);
        }
    });

    // we never save into the heap we read line by line and clear the buffer so we have only one
    // string represente the line and not all the file in memory
    // let file_path = "data/poem.txt";
    // let f = File::open(&file_path).unwrap_or_else(|err| {
    //     eprintln!("Error opening the file : {err}");
    //     process::exit(1);
    // });
    //
    // let mut reader = BufReader::new(f);
    // let mut line: String = String::new();
    //
    // let n_bytes = reader.read_line(&mut line).expect("error!");
    // println!("num bytes : {}\nLine: {}", n_bytes, line);
    // line.clear();

    //
    // let search = Search::new(&config, &content);
    // println!("{:?}", search.result);
}
