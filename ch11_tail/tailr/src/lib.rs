use crate::TakeValue::*;
use clap::{App, Arg};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

static NUM_RE: OnceCell<Regex> = OnceCell::new();

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("tailr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust tail")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .default_value("10"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .takes_value(true)
                .conflicts_with("lines")
                .help("Number of bytes"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Suppress headers"),
        )
        .get_matches();

    let lines = matches
        .value_of("lines")
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;

    let bytes = matches
        .value_of("bytes")
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: lines.unwrap(),
        bytes,
        quiet: matches.is_present("quiet"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let has_multple_files = config.files.len() > 1;

    for (file_num, filename) in config.files.iter().enumerate() {
        match File::open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                if !config.quiet && has_multple_files {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }

                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                let file = BufReader::new(file);
                if let Some(ref n) = config.bytes {
                    print_bytes(file, n, total_bytes)?;
                } else {
                    print_lines(file, &config.lines, total_lines)?;
                };
            }
        }
    }
    Ok(())
}

fn parse_num(val: &str) -> MyResult<TakeValue> {
    let num_re = NUM_RE.get_or_init(|| Regex::new(r"^([+-])?(\d+)$").unwrap());

    match num_re.captures(val) {
        Some(caps) => {
            let sign = caps.get(1).map_or("-", |m| m.as_str());
            let num = format!("{}{}", sign, caps.get(2).unwrap().as_str());
            if let Ok(num) = num.parse() {
                if sign == "+" && num == 0 {
                    Ok(PlusZero)
                } else {
                    Ok(TakeNum(num))
                }
            } else {
                Err(From::from(val))
            }
        }
        _ => Err(From::from(val)),
    }
}

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);

    let mut num_lines: i64 = 0;
    let mut num_bytes: i64 = 0;
    let mut buf = Vec::new();

    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }

        num_lines += 1;
        num_bytes += bytes_read as i64;
        buf.clear();
    }

    Ok((num_lines, num_bytes))
}

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut buf = Vec::new();
        let mut line_num = 0;
        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            if line_num >= start {
                print!("{}", String::from_utf8_lossy(&buf));
            }
            line_num += 1;
            buf.clear();
        }
    }
    Ok(())
}

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> MyResult<()> {
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            print!("{}", String::from_utf8_lossy(&buffer));
        }
    }
    Ok(())
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => {
            if total > 0 {
                Some(0)
            } else {
                None
            }
        }
        TakeNum(num) => {
            let num = *num;
            if num == 0 || total == 0 || num > total {
                None
            } else {
                let start = if num < 0 { total + num } else { num - 1 };
                Some(if start < 0 { 0 } else { start as u64 })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, get_start_index, parse_num, TakeValue::*};

    #[test]
    fn test_parse_num() {
        // すべての整数は負の数として解釈される必要がある
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // 先頭に「+」が付いている場合は正の数として解釈される必要がある
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // 明示的に「-」が付いている場合は負の数として解釈される必要がある
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // ゼロはゼロのまま
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // プラス��ロ特別扱い
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // 境界値のテスト
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // 浮動小数点数は無効
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // 整数でない文字列は無効
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        // 空のファイル（0行/バイト）に対て+0を指定したときはNoneを返す
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // 空でなファイルに対して+0を指定したときは0を返す
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // 0行/バイトを指定した場合はNoneを返す
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // 空のファイルから行/バイトを取得するとNoneを返す
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // ファイルの行数やバイト数を超える位置を取得しようとするとNoneを返す
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // 開始行や開始バイトがファイルの行数やバイト数より小さい場合、
        // 開始行や開始バイトより1小さい値を返す
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // 開始行や開始バイトが負の場合、
        // ファイルの行数/バイト数に開始行/バイトを足した結果を返す
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // 開始行や開始バイトが負、足した結果が0より小さい場合、
        // ファイル全体を表示するために0を返す
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}
