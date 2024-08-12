use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust grep")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Search pattern")
                .required(true),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .multiple(true),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Count occurrences")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("invert-match")
                .short("v")
                .long("invert-match")
                .help("Invert match")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Recursive search")
                .takes_value(false),
        )
        .get_matches();

    let pattern = matches.value_of("pattern").unwrap();
    let regex = RegexBuilder::new(pattern)
        .case_insensitive(matches.is_present("insensitive"))
        .build()
        .map_err(|_e| format!("Invalid pattern \"{}\"", pattern))?;

    Ok(Config {
        pattern: regex,
        files: matches.values_of_lossy("files").unwrap(),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert-match"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let entries = find_files(&config.files, config.recursive);
    let num_entries = entries.len();
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => {
                let header = if num_entries > 1 {
                    format!("{}:", filename)
                } else {
                    String::new()
                };

                match open(&filename) {
                    Err(e) => eprintln!("{}: {}", filename, e),
                    Ok(file) => {
                        let matches = find_lines(file, &config.pattern, config.invert_match);
                        if config.count {
                            println!("{}{}", header, matches.unwrap().len());
                        } else {
                            for v in matches.unwrap() {
                                print!("{}{}", header, v);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut results = Vec::new();

    for path in paths {
        let path = Path::new(path);
        if !path.exists() {
            results.push(Err(format!(
                "{}: No such file or directory ",
                path.to_string_lossy()
            )
            .into()));
            continue;
        }

        if path.is_file() {
            results.push(Ok(path.to_string_lossy().to_string()));
            continue;
        }

        if !path.is_dir() {
            continue; // Skip if it's neither a file nor a directory
        }

        if !recursive {
            results.push(Err(
                format!("{} is a directory", path.to_string_lossy()).into()
            ));
            continue;
        }

        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| match e {
                Err(e) => {
                    eprint!("{}", e);
                    None
                }
                Ok(e) => Some(e),
            })
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();

        results.extend(entries.into_iter().map(Ok));
    }

    results
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn find_lines<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<String>> {
    let mut line = String::new();
    let mut result = vec![];

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if pattern.is_match(&line) != invert_match {
            result.push(line.clone());
        }

        line.clear();
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_files() {
        // 存在することがわかっているファイルを見つけられることを確認する
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // recursiveなしの場合、ディレクトリを拒否する
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // ディレクトリ内の4つのファイルを再帰的に検索できることを確認する
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // 存在しないファイルを表すランダムな文字列を生成する
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // エラーとして不正なファイルを返すことを確認する
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        // 「or」というパターンは「Lorem」という1行にマッチするはず
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        // マッチを反転させた場合、残りの2行にマッチするはず
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // 大文字と小文字を区別しない正規表現
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        // 「Lorem」と「DOLOR」の2行にマッチするはず
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // マッチを反転させた場合、残りの1行にマッチするはず
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
