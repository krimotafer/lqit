use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

use minigrep::lexer::Lexer;

fn main() {
    let user_args: Vec<String> = env::args().collect();

    // we skip the first element that represent the name of the binary
    let _lexer = Lexer::new(&user_args[1..]);

    // we never save into the heap we read line by line and clear the buffer so we have only one
    // string represente the line and not all the file in memory
    let file_path = "poem.txt";
    let f = File::open(&file_path).unwrap_or_else(|err| {
        eprintln!("Error opening the file : {err}");
        process::exit(1);
    });

    let mut reader = BufReader::new(f);
    let mut line: String = String::new();

    let n_bytes = reader.read_line(&mut line).expect("error!");
    println!("num bytes : {}\nLine: {}", n_bytes, line);
    line.clear();

    //
    // let search = Search::new(&config, &content);
    // println!("{:?}", search.result);
}
