use clap::{App, Arg};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

enum Writer {
    Stdout(io::Stdout),
    File(File),
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::Stdout(out) => out.write(buf),
            Writer::File(file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::Stdout(out) => out.flush(),
            Writer::File(file) => file.flush(),
        }
    }
}

fn write_data(writer: &mut Writer, data: &str) -> io::Result<()> {
    write!(writer, "{}", data)
}

fn output(out_file: Option<String>, content: &str) -> io::Result<()> {
    match out_file {
        Some(file_path) => {
            let file = File::create(file_path)?;
            let mut writer = Writer::File(file);
            write_data(&mut writer, content)?;
            writer.flush()?;
        }
        None => {
            let mut stdout = Writer::Stdout(io::stdout());
            write_data(&mut stdout, content)?;
            stdout.flush()?;
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
        .about("Rust uniq")
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .help("Input file")
                .required(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .help("Output file"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Show counts")
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    Ok(Config {
        in_file: matches.value_of_lossy("in_file").unwrap().to_string(),
        out_file: matches.value_of("out_file").map(String::from),
        count: matches.is_present("count"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;

    let mut line = String::new();
    let mut map: BTreeMap<String, u32> = BTreeMap::new();

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        //        let trimmed_line = line.trim_end();
        *map.entry(line.to_string()).or_insert(0) += 1;

        line.clear();
    }

    let mut output_lines: Vec<String> = Vec::new();

    for (k, v) in &map {
        if config.count {
            output_lines.push(format!("{:>4} {}", v, k));
        } else {
            output_lines.push(format!("{}", k));
        }
    }

    //    output_lines.push(format!("{}", "\n"));
    let _ = output(config.out_file, &output_lines.join("\n"));

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
