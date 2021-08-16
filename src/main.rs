mod accounts;
mod record;

use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use rev_lines::RevLines;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
pub enum MyErrors {
    CannotOpenFile(std::io::Error),
    CannotReadFileInReverse(std::io::Error),
    FileHasNoLines,
    LineWithoutCommas,
    CannotParseDate(chrono::ParseError),
    CannotCreateTime,
}

fn main() {
    for (tick, filename) in accounts::get_accounts().iter() {
        println!("{} {}", tick, filename);

        let t = read_last_date_from_file(filename).unwrap();

        println!("{}", t);
    }

    let txn = record::Record {
        date: "2021-08-16".to_string(),
        payee: "FIDELITY".to_string(),
        category: "Investment Status".to_string(),
        note: "Foo".to_string(),
        amount: 9.28,
    };

    println!("{}", txn);
}

fn read_last_date_from_file(filename: &str) -> Result<NaiveDateTime, MyErrors> {
    let file = match File::open(filename) {
        Ok(res) => res,
        Err(err) => return Err(MyErrors::CannotOpenFile(err)),
    };

    let mut rev_lines = match RevLines::new(BufReader::new(file)) {
        Ok(res) => res,
        Err(err) => return Err(MyErrors::CannotReadFileInReverse(err)),
    };

    let last_line = match rev_lines.next() {
        Some(res) => res,
        None => return Err(MyErrors::FileHasNoLines),
    };

    let date = match last_line.split(",").nth(0) {
        Some(res) => res,
        None => return Err(MyErrors::LineWithoutCommas),
    };

    let d = match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(res) => res,
        Err(err) => return Err(MyErrors::CannotParseDate(err)),
    };

    let t = match NaiveTime::from_hms_opt(9, 0, 0) {
        Some(res) => res,
        None => return Err(MyErrors::CannotCreateTime),
    };

    // TODO: Add one day to <date> to ignore already synced data.

    Ok(d.and_time(t))
}
