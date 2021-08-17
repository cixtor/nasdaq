mod accounts;
mod record;

use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use rev_lines::RevLines;
use std::fs::File;
use std::io::BufReader;

use crate::record::Record;

#[derive(Debug)]
pub enum MyErrors {
    CannotOpenFile(std::io::Error),
    CannotReadFileInReverse(std::io::Error),
    FileHasNoLines,
    LineWithoutCommas,
    CannotParseDate(chrono::ParseError),
    CannotCreateTime,
    IgnoreCSVHeader,
    NotEnoughColumns,
    MissingFirstColumn,
}

fn main() {
    for (tick, filename) in accounts::get_accounts().iter() {
        println!("{} {}", tick, filename);

        let t = read_last_date_from_file(filename).unwrap();

        println!("{}", t);
    }

    let txn = parse_line(
        "FOO",
        1,
        "2021-08-17,240.570007,255.330002,239.860001,255.139999,255.139999,47553800",
    )
    .unwrap();

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

fn parse_line(tick: &str, line_number: usize, line: &str) -> Result<Record, MyErrors> {
    if line_number == 0 {
        return Err(MyErrors::IgnoreCSVHeader);
    }

    let cols: Vec<&str> = line.split(",").collect();

    if cols.len() < 7 {
        return Err(MyErrors::NotEnoughColumns);
    }

    let date = match cols.first() {
        Some(res) => res,
        None => return Err(MyErrors::MissingFirstColumn),
    };

    let mut note: Vec<String> = Vec::new();
    note.push(tick.to_string());
    note.push(format!("Open: {}", cols[1]));
    note.push(format!("High: {}", cols[2]));
    note.push(format!("Low: {}", cols[3]));
    note.push(format!("Close: {}", cols[4]));
    note.push(format!("Adj Close: {}", cols[5]));
    note.push(format!("Volume: {}", cols[6]));

    Ok(Record {
        date: date.to_string(),
        payee: "FIDELITY".to_string(),
        category: "Investment Status".to_string(),
        note: note.join(" @ "),
        amount: 0.00,
    })
}
