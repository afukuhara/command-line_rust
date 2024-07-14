use crate::Extract::*;
use clap::{App, Arg};
use std::{error::Error, ops::Range};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    // extract: Extract,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust cut")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .required(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("b")
                .long("bytes")
                .help("Selected bytes")
                .takes_value(true)
                .multiple(true)
                .conflicts_with("chars"),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .short("c")
                .long("chars")
                .help("Selected characters")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("delim")
                .value_name("DELEMITER")
                .short("d")
                .long("delim")
                .help("Field delimiter")
                .takes_value(true)
                .default_value("\t"),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .short("f")
                .long("fields")
                .help("Selected fields")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap_or_default(),
        delimiter: matches
            .value_of("delim")
            .map(|c| c.chars().next().unwrap() as u8)
            .unwrap_or(b','),
        // extract: matches.values_of("fields").map(Extract::from).unwrap_or_default(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    unimplemented!();
}
