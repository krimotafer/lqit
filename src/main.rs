use std::env;
use std::fs;
use std::process;

#[derive(Debug)]
struct Config {
    query: String,
    file_path: String,
}

impl Config {
    fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => {
                return Err("No query in the arguments");
            }
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => {
                return Err("No file path.");
            }
        };

        Ok(Config { query, file_path })
    }
}

#[derive(Debug)]
struct Search<'a> {
    config: &'a Config,
    content: &'a str,
    pub result: Vec<&'a str>,
}

impl<'a> Search<'a> {
    pub fn new(config: &'a Config, content: &'a str) -> Self {
        let result: Vec<&str> = content
            .lines()
            .filter(|line| line.contains(&config.query))
            .collect();

        Self {
            config,
            content,
            result,
        }
    }
}

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    let content = fs::read_to_string(&config.file_path).unwrap_or_else(|err| {
        eprintln!("Error reading the file : {err}");
        process::exit(1);
    });

    let search = Search::new(&config, &content);
    dbg!(search);
}
