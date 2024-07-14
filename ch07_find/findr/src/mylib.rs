use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::{DirEntry, WalkDir};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust find")
        .arg(
            Arg::with_name("path")
                .value_name("PATH")
                .help("Search paths")
                .default_value(".")
                .multiple(true),
        )
        .arg(
            Arg::with_name("names")
                .value_name("NAME")
                .short("n")
                .long("name")
                .help("Name")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("types")
                .value_name("TYPE")
                .short("t")
                .long("type")
                .help("Entry type")
                .possible_values(&["f", "d", "l"])
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let names = matches
        .values_of_lossy("names")
        .map(|names| {
            names
                .into_iter()
                .map(|name| Regex::new(&name).map_err(|_e| format!("Invalid --name '{}'", name)))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    let entry_types = matches
        .values_of_lossy("types")
        .map(|vals| {
            vals.iter()
                .map(|val| match val.as_str() {
                    "d" => Dir,
                    "f" => File,
                    "l" => Link,
                    _ => unreachable!("Invalid type"),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Config {
        paths: matches.values_of_lossy("path").unwrap(),
        names,
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:#?}", config);

    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprint!("{}", e),
                Ok(entry) => {
                    if matches_entry_type(&entry, &config.entry_types)
                        && matches_name(&entry, &config.names)
                    {
                        println!("{}", entry.path().display());
                    }
                }
            }
        }
    }

    Ok(())
}

fn matches_entry_type(entry: &DirEntry, entry_types: &[EntryType]) -> bool {
    let e_type = entry.file_type();
    entry_types.is_empty()
        || entry_types.iter().any(|t| match t {
            EntryType::File => e_type.is_file(),
            EntryType::Dir => e_type.is_dir(),
            EntryType::Link => e_type.is_symlink(),
        })
}

fn matches_name(entry: &DirEntry, names: &[Regex]) -> bool {
    names.is_empty()
        || names
            .iter()
            .any(|regex| regex.is_match(entry.file_name().to_str().unwrap_or_default()))
}
