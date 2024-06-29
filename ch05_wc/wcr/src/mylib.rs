use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust wc")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("File(s) to input")
                .required(true)
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .help("Sohw line count")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help("Show word count")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help("Show byte count")
                .takes_value(false)
                .conflicts_with("chars")
                .required(false),
        )
        .arg(
            Arg::with_name("chars")
                .short("m")
                .long("chars")
                .help("Show character count")
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    if [lines, words, bytes, chars].iter().all(|v| v == &false) {
        lines = true;
        words = true;
        bytes = true
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        chars,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total = FileInfo {
        num_lines: 0,
        num_words: 0,
        num_bytes: 0,
        num_chars: 0,
    };

    let filenum = config.files.len();
    let total_config = config.clone();
    let files = config.files.clone();

    for filename in files {
        match open(&filename) {
            Err(err) => eprint!("Failed to open {}: {}", filename, err),
            Ok(reader) => {
                let fileinfo = count(reader).unwrap();
                print_result(&config, &fileinfo, &filename);

                total = plus_fileinfo(&total, &fileinfo)?;
            }
        }
    }

    if filenum > 1 {
        print_result(&total_config, &total, "total");
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn print_result(config: &Config, fileinfo: &FileInfo, filename: &str) {
    let mut output = String::new();

    if config.lines {
        output.push_str(&format!("{:>8}", fileinfo.num_lines));
    }
    if config.words {
        output.push_str(&format!("{:>8}", fileinfo.num_words));
    }
    if config.bytes {
        output.push_str(&format!("{:>8}", fileinfo.num_bytes));
    }

    if config.chars {
        output.push_str(&format!("{:>8}", fileinfo.num_chars));
    }

    if !output.is_empty() {
        println!("{} {}", output, filename);
    }
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let reader = BufReader::new(file.fill_buf()?);
    let num_lines = count_lines(reader);

    let reader = BufReader::new(file.fill_buf()?);
    let num_words = count_words(reader);

    let reader = BufReader::new(file.fill_buf()?);
    let num_bytes = count_bytes(reader);

    let reader = BufReader::new(file.fill_buf()?);
    let num_chars = count_chars(reader);

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

fn plus_fileinfo(a: &FileInfo, b: &FileInfo) -> MyResult<FileInfo> {
    Ok(FileInfo {
        num_lines: a.num_lines + b.num_lines,
        num_words: a.num_words + b.num_words,
        num_bytes: a.num_bytes + b.num_bytes,
        num_chars: a.num_chars + b.num_bytes,
    })
}

fn count_lines(reader: impl BufRead) -> usize {
    reader.lines().count()
}

fn count_words(reader: impl BufRead) -> usize {
    reader
        .lines()
        .map(|l| l.unwrap().split_ascii_whitespace().count())
        .sum()
}

fn count_bytes(mut reader: impl BufRead) -> usize {
    let mut buffer = Vec::new();
    let bytes_read = reader.read_to_end(&mut buffer);
    bytes_read.unwrap()
}

fn count_chars(reader: impl BufRead) -> usize {
    reader.lines().map(|l| l.unwrap().chars().count()).sum()
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };

        let info = count(Cursor::new(text));

        assert!(info.is_ok());
        assert_eq!(info.unwrap(), expected)
    }
}
