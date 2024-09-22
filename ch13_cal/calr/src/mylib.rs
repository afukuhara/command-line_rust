use chrono::Month;
use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};
use std::error::Error;
use std::str::FromStr;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

type MyResult<T> = Result<T, Box<dyn Error>>;

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("calr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rust cal")
        .arg(Arg::with_name("year").help("Year (1-9999)").index(1))
        .arg(
            Arg::with_name("month")
                .short("m")
                .long("month")
                .help("Month name or number (1-12)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("year_flag")
                .short("y")
                .long("year")
                .help("Show whole current year"),
        )
        .get_matches();

    let today = Local::now().date_naive();
    let mut month = None;
    let mut year = today.year();
    if let Some(year_str) = matches.value_of("year") {
        year = year_str.parse::<i32>().unwrap();
    }

    if let Some(month_str) = matches.value_of("month") {
        month = matches
            .value_of("month")
            .map(parse_month)
            .transpose()
            .map_err(|e| format!("{}", e))?;
    }

    Ok(Config { month, year, today })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    match val.parse::<T>() {
        Ok(num) => Ok(num),
        Err(_) => Err(format!("Invalid integer \"{}\"", val).into()),
    }
}

fn parse_year(year: &str) -> MyResult<i32> {
    match parse_int(year) {
        Ok(year_num) => {
            if year_num >= 1 && year_num <= 9999 {
                return Ok(year_num);
            } else {
                return Err(format!("year \"{}\" not in the range 1 through 9999", year).into());
            }
        }
        Err(e) => Err(e),
    }
}

fn parse_month(month: &str) -> MyResult<u32> {
    if let Ok(month_num) = parse_int(month) {
        if month_num >= 1 && month_num <= 12 {
            return Ok(month_num);
        } else {
            return Err(format!("month \"{}\" not in the range 1 through 12", month).into());
        }
    } else {
        match Month::from_str(month) {
            Ok(month) => Ok(month.number_from_month()),
            Err(_) => Err(format!("Invalid month \"{}\"", month).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_int, parse_month, parse_year};

    #[test]
    fn test_parse_int() {
        // 正の整数をusizeとして解析する
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        // 負の整数をi32として解析する
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        // 数字以外の文字列を解析すると失敗する
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }
}
