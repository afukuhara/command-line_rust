use clap::{App, Arg};
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::path::PathBuf;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Fortune {
    source: String,
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust fortune")
        .arg(
            Arg::with_name("sources")
                .value_name("FILE")
                .help("Input files or directories")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive pattern matching")
                .takes_value(false),
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

    let pattern = matches
        .value_of("pattern")
        .map(|val| {
            RegexBuilder::new(val)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid --pattern \"{}\"", val))
        })
        .transpose()?;

    Ok(Config {
        sources: matches.values_of_lossy("sources").unwrap(),
        seed: matches.value_of("seed").map(parse_u64).transpose()?,
        pattern,
    })
}

fn parse_u64(val: &str) -> MyResult<u64> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let mut entries = paths
        .iter()
        .flat_map(|path| {
            WalkDir::new(path)
                .into_iter()
                .map(|entry| entry.map(|e| e.into_path()))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|p| p.is_file())
        .collect::<Vec<_>>();

    entries.sort();
    entries.dedup();
    Ok(entries)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut fortunes = Vec::new();

    for path in paths {
        let content = std::fs::read_to_string(path)?;
        let mut tmp_fortunes: Vec<Fortune> = content
            .split('%')
            .map(|t| Fortune {
                source: path.to_string_lossy().into_owned(),
                text: t.trim().to_string(),
            })
            .filter(|t| !t.text.is_empty())
            .collect();
        fortunes.append(&mut tmp_fortunes);
    }

    Ok(fortunes)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    let mut rng: StdRng = if let Some(seed) = seed {
        SeedableRng::seed_from_u64(seed)
    } else {
        SeedableRng::from_entropy()
    };

    if fortunes.is_empty() {
        None
    } else {
        Some(fortunes.choose(&mut rng)?.text.clone())
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    let mut found = false;

    if let Some(pattern) = config.pattern {
        for fortune in fortunes.iter() {
            if pattern.is_match(&fortune.text) {
                println!("{}", fortune.text);
                found = true;
            }
        }

        if !found {
            eprintln!("No fortunes found");
        }
    } else {
        if let Some(fortune) = pick_fortune(&fortunes, config.seed) {
            println!("{}", fortune);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{find_files, parse_u64, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;

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

    #[test]
    fn test_find_files() {
        // 存在するファイルを検索できることを確認する
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // 存在しないファイルの検索には失敗する
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // 拡張子が「.dat」以外の入力ファイルをすべて検索する
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // ファイル数とファイルの順番を確認する
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // 複数のソースに対するテストをする。
        // パスは重複なしでソートされた状態でなければならない
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // 入力ファイルが1つだけの場合
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        println!("{:#?}", res);

        if let Ok(fortunes) = res {
            // 数が正しいこととソートされていることを確認する
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // 入力ファイルが複数の場合
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Fortuneのスライスを作成
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // シードを与えて引用句を1つ選択
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
