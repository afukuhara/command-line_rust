use chrono::{DateTime, Local};
use clap::{App, Arg};
use std::fs;
use std::path::PathBuf;
use std::{error::Error, fs::Metadata, os::unix::fs::MetadataExt};
use tabular::{Row, Table};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("lsr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust ls")
        .arg(
            Arg::with_name("paths")
                .value_name("PATH")
                .help("Files and/or directories")
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Show all files"),
        )
        .arg(
            Arg::with_name("long")
                .short("l")
                .long("long")
                .help("Long listing"),
        )
        .get_matches();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        long: matches.is_present("long"),
        show_hidden: matches.is_present("all"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let paths = find_files(&config.paths, config.show_hidden)?;
    for path in paths {
        println!("{}", path.display());
    }
    Ok(())
}

fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = vec![];

    let show_entry = |path: PathBuf| -> Option<PathBuf> {
        if show_hidden {
            Some(path)
        } else if path.file_name().and_then(|s| s.to_str())?.starts_with(".") {
            None
        } else {
            Some(path)
        }
    };

    for path in paths {
        match fs::metadata(path) {
            Err(e) => return Err(format!("{}: {}", path, e).into()),
            Ok(entry) => {
                if entry.is_dir() {
                    let entries = fs::read_dir(path).unwrap();
                    for entry in entries {
                        if let Some(entry) = show_entry(entry.unwrap().path()) {
                            files.push(entry);
                        }
                    }
                } else if entry.is_file() {
                    if show_hidden || !path.starts_with(".") {
                        files.push(path.into());
                    }
                }
            }
        }
    }

    Ok(files)
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:<}  {:<}  {:<}";
    let mut table = Table::new(fmt);

    let entry_type = |metadata: &Metadata| -> String {
        if metadata.is_dir() {
            "d".to_string()
        } else if metadata.is_file() {
            "-".to_string()
        } else {
            "?".to_string()
        }
    };

    let entry_timestamp = |metadata: &Metadata| -> String {
        if let Ok(modified_time) = metadata.modified() {
            let datetime: DateTime<Local> = modified_time.into();
            datetime.format("%Y-%m-%d").to_string()
        } else {
            "更新日の取得に失敗しました".to_string()
        }
    };

    for path in paths {
        let metadata = path.metadata().unwrap();
        table.add_row(
            Row::new()
                .with_cell(entry_type(&metadata))
                .with_cell(format_mode(metadata.mode()))
                .with_cell(format!("{:o}", metadata.nlink()))
                .with_cell(metadata.uid().to_string())
                .with_cell(metadata.gid().to_string())
                .with_cell(metadata.size().to_string())
                .with_cell(entry_timestamp(&metadata))
                .with_cell(path.display().to_string()),
        );
    }

    println!("table: {}", table);
    Ok(format!("{}", table))
}

/// 0o751のような8進数でファイルモードを指定すると、
/// 「rwxr-x--x」のような文字列を返す。
fn format_mode(mode: u32) -> String {
    let user = (mode / 8_u32.pow(2)) % 8;
    let group = (mode / 8_u32.pow(1)) % 8;
    let other = mode % 8;

    // println!(
    //     "mode: {:o}, user: {:o}, group: {:o}, other: {:o}",
    //     mode, user, group, other
    // );

    let convert_to_str = |n: u32| -> &str {
        match n {
            0 => "---",
            1 => "--x",
            2 => "-w-",
            3 => "-wx",
            4 => "r--",
            5 => "r-x",
            6 => "rw-",
            7 => "rwx",
            _ => "???",
        }
    };

    format!(
        "{}{}{}",
        convert_to_str(user),
        convert_to_str(group),
        convert_to_str(other)
    )
}

#[cfg(test)]
mod test {
    use super::{find_files, format_mode, format_output};
    use std::path::PathBuf;

    // テストのためのヘルパー関数
    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        println!("parts: {:?}", parts);
        assert!(parts.len() > 0 && parts.len() <= 10);

        let perms = parts.get(0).unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_find_files() {
        // ディレクトリにある隠しエントリ以外のエントリを検索する
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // 存在するファイルは、隠しファイルであっても検索できるようにする
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // 複数のパスを与えてテストする
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // ディレクトリにあるすべてのエントリを検索する
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(&line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            &empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(&dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }
}
