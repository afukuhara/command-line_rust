use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate, Weekday};
use clap::{App, Arg};
use std::error::Error;
use std::str::FromStr;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

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
        .author("Arinobu Fukuhara <afukuhara@gmail.com>")
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
            Arg::with_name("show_current_year")
                .short("y")
                .long("year")
                .help("Show whole current year")
                .conflicts_with_all(&["month", "year"])
                .takes_value(false),
        )
        .get_matches();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches.value_of("year").map(parse_year).transpose()?;
    let today = Local::now().date_naive();

    if matches.is_present("show_current_year") {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }

    Ok(Config {
        month,
        year: year.unwrap_or_else(|| today.year()),
        today,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    if config.month.is_none() {
        println!("{:>32}", config.year);
        let mut tmp_month = vec![];
        for month in 1..=12 {
            let month_cal = format_month(config.year, month, false, config.today);

            if month % 3 == 1 {
                tmp_month = month_cal;
            } else {
                tmp_month = tmp_month
                    .iter()
                    .zip(month_cal.iter())
                    .map(|(a, b)| format!("{}{}", a, b))
                    .collect();

                if month % 3 == 0 {
                    println!("{}", tmp_month.join("\n"));
                    println!("");
                }
            }
        }
    } else {
        let month = config.month.unwrap();
        let year = config.year;
        let month_cal = format_month(year, month, true, config.today);
        println!("{}", month_cal.join("\n"));
    }
    Ok(())
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

fn parse_year(year: &str) -> MyResult<i32> {
    parse_int(year).and_then(|num| {
        if (1..=9999).contains(&num) {
            Ok(num)
        } else {
            Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
        }
    })
}

fn parse_month(month: &str) -> MyResult<u32> {
    match parse_int(month) {
        Ok(month_num) => {
            if (1..=12).contains(&month_num) {
                Ok(month_num)
            } else {
                Err(format!("month \"{}\" not in the range 1 through 12", month).into())
            }
        }
        _ => {
            let lower = month.to_lowercase();
            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(&lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();

            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                Err(format!("Invalid month \"{}\"", month).into())
            }
        }
    }
}

fn week_of_month(date: NaiveDate) -> i32 {
    let day_of_month = date.day() as i32;
    let first_day_of_month = date.with_day(1).unwrap();
    let first_sunday_of_month = first_day_of_month
        .iter_days()
        .find(|d| d.weekday() == Weekday::Sun)
        .unwrap();
    let days_to_first_sunday: i32 = day_of_month - first_sunday_of_month.day() as i32;
    let week_of_month = ((days_to_first_sunday - 7) as f64 / 7.0).floor() as i32 + 2;

    week_of_month
}

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let last_day = last_day_in_month(year, month);
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    let header_text = if print_year {
        format!("{} {}", MONTH_NAMES[month as usize - 1].to_string(), year)
    } else {
        format!("{}", MONTH_NAMES[month as usize - 1].to_string())
    };
    let header = format!("{:^20}", header_text);

    let headers = vec![header, "Su Mo Tu We Th Fr Sa".to_string()];
    let mut cal: Vec<Vec<Option<NaiveDate>>> = vec![vec![None; 7]; 6];

    for d in first_day.iter_days().take(last_day.day() as usize) {
        let week_of_month = week_of_month(d);
        let weekday = d.weekday().num_days_from_sunday();
        cal[week_of_month as usize][weekday as usize] = Some(d);
    }

    let mut calendar = headers;
    let blank = "  ".to_string();
    let style = Style::new().reverse();
    for week in cal {
        let week_str = week
            .iter()
            .map(|day| {
                if day.is_none() {
                    blank.clone()
                } else {
                    let this_day = day.unwrap();
                    if this_day == today {
                        format!("{:>2}", style.paint(this_day.day().to_string()))
                    } else {
                        format!("{:>2}", this_day.day().to_string())
                    }
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        calendar.push(format!("{:<20}", week_str));
    }

    calendar.iter().map(|c| format!("{}  ", c)).collect()
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    // 次の月の1日を作成
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    // 1日前に戻って当月の最終日を取得
    next_month.unwrap().pred_opt().unwrap()
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, parse_int, parse_month, parse_year};
    use chrono::NaiveDate;

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

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }
}
