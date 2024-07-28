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
    extract: Extract,
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
                .conflicts_with("chars")
                .conflicts_with("fields")
                .validator(is_number),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .short("c")
                .long("chars")
                .help("Selected characters")
                .takes_value(true)
                .conflicts_with("bytes")
                .conflicts_with("fields")
                .multiple(true)
                .validator(is_number),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .short("f")
                .long("fields")
                .help("Selected fields")
                .takes_value(true)
                .conflicts_with("chars")
                .conflicts_with("bytes")
                .multiple(true),
        )
        .arg(
            Arg::with_name("delim")
                .value_name("DELEMITER")
                .short("d")
                .long("delim")
                .help("Field delimiter")
                .takes_value(true)
                .default_value("\t")
                .validator(is_one_byte),
        )
        .get_matches();

    let position_list = match parse_pos(
        matches
            .value_of_lossy("fields")
            .map(String::from)
            .unwrap()
            .as_str(),
    ) {
        Ok(config) => config,
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    };

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap_or_default(),
        delimiter: matches
            .value_of("delim")
            .map(|c| c.chars().next().unwrap() as u8)
            .unwrap_or(b','),
        extract: Fields(position_list),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

fn is_one_byte(v: String) -> Result<(), String> {
    if v.len() == 1 && v.as_bytes()[0] <= 8 {
        Ok(())
    } else {
        Err(format!("--delim \"{}\" must be a single byte", v))
    }
}

fn is_number(val: String) -> Result<(), String> {
    val.parse::<u32>()
        .map(|_| ())
        .map_err(|_| format!("illegal list value: \"{}\"", val))
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    if range.is_empty() {
        eprintln!("Empty!");
        return Err(From::from("Range cannot be empty!"));
    }

    range
        .split(',')
        .map(|part| {
            if part.contains('-') {
                convert_to_list(part)
            } else {
                let n = validate_element(part);
                match n {
                    Ok(0) => Err(From::from(format!("illegal list value: \"0\""))),
                    Ok(n) => Ok(Range {
                        start: n - 1,
                        end: n,
                    }),
                    Err(e) => Err(e),
                }
            }
        })
        .collect::<MyResult<PositionList>>()
}

fn convert_to_list(v: &str) -> MyResult<Range<usize>> {
    let mut nums = v.split('-');

    let start = validate_element(nums.next().unwrap());
    let end = validate_element(nums.next().unwrap());

    if nums.next().is_some() {
        return Err(From::from(format!("Too many list elements")));
    }

    match (start, end) {
        (Ok(0), _) | (_, Ok(0)) => Err(From::from(format!("illegal list value: \"0\""))),
        (Ok(start), Ok(end)) if start < end => Ok(Range {
            start: start - 1,
            end: end,
        }),
        (Ok(start), Ok(end)) => Err(From::from(format!(
            "First number in range ({}) must be lower than second number ({})",
            start, end
        ))),
        (Err(e), _) | (_, Err(e)) => Err(From::from(format!("illegal list value: \"{}\"", v))),
    }
}

fn validate_element(v: &str) -> MyResult<usize> {
    if v.contains("+") {
        eprintln!("illegal list value: \"{}\"", v);
        Err(From::from(format!("illegal list value: \"{}\"", v)))
    } else {
        let result = v.parse::<usize>();
        match result {
            Ok(number) => Ok(number),
            Err(e) => Err(From::from(format!("illegal list value: \"{}\"", v))),
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::parse_pos;

    #[test]
    fn test_parse_pos() {
        // 空文字列はエラー
        assert!(parse_pos("").is_err());

        // ゼロはエラー
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        // 数字の前に「+」が付く場合はエラー
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);

        // 数字以外はエラー
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);

        // エラーになる範囲
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // 最初の数字は2番目より小さい必要がある
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // 以下のケースは受け入れられる
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}
