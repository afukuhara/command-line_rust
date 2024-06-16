use clap::{App, Arg};
use std::collections::btree_map::Range;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust head")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("File(s) to input")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("c")
                .long("bytes")
                .help("Number of bytes")
                .conflicts_with("lines")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("lines")
                .value_name("LINES")
                .short("n")
                .long("lines")
                .help("Number of lines [default: 10]")
                .takes_value(true)
                .default_value("10"),
        )
        .get_matches();

    let lines = matches
        .value_of("lines")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| {
            format!(
                "error: invalid value '{}' for '--lines <LINES>': invalid digit found in string",
                e
            )
        })?;

    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| {
            format!(
                "error: invalid value '{}' for '--bytes <BYTES>': invalid digit found in string",
                e
            )
        })?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: lines.unwrap(),
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let files = config.files;
    let has_multple_files = files.len() > 1;

    let mut is_first = true;

    for filename in files {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(reader) => {
                if has_multple_files {
                    if !is_first {
                        println!("");
                    }
                    println!("==> {} <==", filename)
                }
                match config.bytes {
                    Some(num) => {
                        let _ = read_file_with_bytes(reader, num);
                        // println!("{}", content);
                    }
                    None => {
                        let _ = read_lines(reader, config.lines);
                        // println!("{}", content);
                    }
                }

                is_first = false;
            }
        }
    }

    Ok(())
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn read_file_with_bytes(mut reader: Box<dyn BufRead>, bytes: usize) -> MyResult<()> {
    let mut buffer = vec![0; bytes];
    let mut bytes_read = 0;

    while bytes_read < bytes {
        let read_count = reader.read(&mut buffer[bytes_read..])?;
        if read_count == 0 {
            break;
        }
        bytes_read += read_count;
    }

    let content = String::from_utf8_lossy(&buffer[..bytes_read]);
    print!("{}", content);

    Ok(())
}

fn read_lines(reader: Box<dyn BufRead>, line_num: usize) -> MyResult<()> {
    let mut i = 1;
    for line in reader.lines() {
        if i > line_num {
            break;
        }

        println!("{}", line.unwrap());
        i += 1;
    }
    Ok(())
}

fn open_byte(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}
