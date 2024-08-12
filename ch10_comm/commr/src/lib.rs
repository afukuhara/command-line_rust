use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("commr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust comm")
        .arg(
            Arg::with_name("file1")
                .help("Input file 1")
                .required(true)
                .takes_value(true)
                .index(1),
        )
        .arg(
            Arg::with_name("file2")
                .help("Input file 2")
                .required(true)
                .takes_value(true)
                .index(2),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .help("Case-insensitive comparison of lines")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("suppress1")
                .short("1")
                .help("Suppress printing of column 1")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("suppress2")
                .short("2")
                .help("Suppress printing of column 2")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("suppress3")
                .short("3")
                .help("Suppress printing of column 3")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("delimiter")
                .short("d")
                .long("output-delimiter")
                .value_name("DELIM")
                .help("Output delimiter")
                .default_value("\t")
                .takes_value(true),
        )
        .get_matches();

    Ok(Config {
        file1: matches.value_of_lossy("file1").unwrap().to_string(),
        file2: matches.value_of_lossy("file2").unwrap().to_string(),
        show_col1: matches.is_present("suppress1"),
        show_col2: matches.is_present("suppress2"),
        show_col3: matches.is_present("suppress3"),
        insensitive: matches.is_present("insensitive"),
        delimiter: matches.value_of_lossy("delimiter").unwrap().to_string(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);

    let file1 = &config.file1;
    let file2 = &config.file2;

    if file1 == "-" && file2 == "-" {
        return Err(From::from("Both input files cannot be STDIN (\"-\")"));
    }

    let _file1 = open(file1)?;
    let _file2 = open(file2)?;
    println!("Opened {} and {}", file1, file2);

    let mut file1_lines = file1.lines();
    let mut file2_lines = file2.lines();

    match (file1_lines.next(), file2_lines.next()) {
        (Some(line1), Some(line2)) => {
            if line1 == line2 {
                println!("{}", line1);
            }
            else if line1 < line2 {
                println!("{}", line1);
            }
            else if line1 > line2    {
                println!("{}", line2);
            }
        }
        _ => {
            println!("EOF");
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
