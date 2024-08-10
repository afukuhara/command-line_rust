use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::fs;
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
        .map_err(|e| format!("Invalid pattern \"{}\"", pattern))?;

    Ok(Config {
        pattern: regex,
        files: matches.values_of_lossy("files").unwrap(),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert-match"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("pattern \"{}\"", config.pattern);

    let entries = find_files(&config.files, config.recursive);
    for entry in entries {
        match entry {
            Ok(file) => println!("file \"{}\"", file),
            Err(e) => eprintln!("{}", e),
        }
    }
    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut results = Vec::new();

    for path in paths {
        let path = Path::new(path);
        if !path.exists() {
            results.push(Err(
                format!("{} does not exist", path.to_string_lossy()).into()
            ));
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

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};

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
}
