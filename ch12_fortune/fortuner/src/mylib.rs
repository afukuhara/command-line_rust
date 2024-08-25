use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust fortune")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input files or directories")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive pattern matching"),
        )
        .arg(
            Arg::with_name("pattern")
                .short("m")
                .long("pattern")
                .value_name("PATTERN")
                .help("Pattern"),
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("SEED")
                .help("Random seed"),
        )
        .get_matches();

    let pattern_str = matches.value_of("pattern").unwrap();
    let pattern = RegexBuilder::new(pattern_str)
        .case_insensitive(matches.is_present("insensitive"))
        .build()
        .map_err(|_e| format!("Invalid pattern \"{}\"", pattern_str))?;

    let seed = matches
        .value_of("seed")
        .map(parse_u64)
        .transpose()
        .map_err(|e| format!("\"{}\" not a valid integer", e))?;

    Ok(Config {
        sources: matches.values_of_lossy("files").unwrap(),
        seed,
        pattern: Some(pattern),
    })
}

fn parse_u64(val: &str) -> MyResult<u64> {
    match val.parse() {
        Ok(n) => Ok(n),
        _ => Err(format!("\"{}\" not a valid integer", val).into()),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_u64;

    #[test]
    fn test_parse_u64() {
        let res = parse_u64("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "\"a\" not a valid integer");

        let res = parse_u64("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = parse_u64("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }
}
