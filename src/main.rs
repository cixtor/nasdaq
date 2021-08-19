mod accounts;
mod record;

use chrono::Local;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use rev_lines::RevLines;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Write;

use crate::record::Record;

#[derive(Debug)]
pub enum MyErrors {
    CannotOpenFile(std::io::Error),
    CannotWriteFile(std::io::Error),
    CannotReadFileInReverse(std::io::Error),
    FileHasNoLines,
    LineWithoutCommas,
    CannotParseDate(chrono::ParseError),
    CannotCreateTime,
    CannotParseTimestamp,
    CannotDownloadData(reqwest::Error),
    NasdaqNotOKResponse,
    CannotReadResponse(reqwest::Error),
    IgnoreCSVHeader,
    NotEnoughColumns,
    MissingFirstColumn,
}

fn main() {
    for (tick, filename) in accounts::get_accounts().iter() {
        if let Err(err) = sync_nasdaq_data(tick, filename) {
            println!("{:?}", err);
        }
    }
}

fn sync_nasdaq_data(tick: &str, filename: &str) -> Result<(), MyErrors> {
    println!("{} {}", tick, filename);

    let t = match read_last_date_from_file(filename) {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    let records = match download_stock_data(tick, t) {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    if records.len() == 0 {
        // No need to open the file.
        return Ok(());
    }

    let mut file = match OpenOptions::new().append(true).open(filename) {
        Ok(res) => res,
        Err(err) => return Err(MyErrors::CannotWriteFile(err)),
    };

    for txn in records {
        println!("{}", txn);

        if let Err(err) = file.write(format!("{}\n", txn).as_bytes()) {
            println!("{} {}", tick, err);
            break;
        }
    }

    Ok(())
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

    let date_time = d.and_time(t);
    let next_day_ts = (date_time.timestamp() + 86400) * 1000;
    let next_day_dt = match NaiveDateTime::from_timestamp_millis(next_day_ts) {
        Some(res) => res,
        None => return Err(MyErrors::CannotParseTimestamp),
    };

    Ok(next_day_dt)
}

// Date,Open,High,Low,Close,Adj Close,Volume
// 2021-11-28,246.080002,246.649994,240.800003,241.759995,241.759995,24778200
// 2021-11-29,241.399994,242.789993,238.210007,240.330002,240.330002,17956300
// 2021-11-30,240.570007,255.330002,239.860001,255.139999,255.139999,47553800
#[tokio::main]
async fn download_stock_data(tick: &str, t: NaiveDateTime) -> Result<Vec<Record>, MyErrors> {
    let period1 = t.timestamp();
    let period2 = Local::now().timestamp();
    let target = format!("https://query1.finance.yahoo.com/v7/finance/download/{}?period1={}&period2={}&interval=1d&events=history&includeAdjustedClose=true", tick, period1, period2);

    let res = match reqwest::get(target).await {
        Ok(res) => res,
        Err(err) => return Err(MyErrors::CannotDownloadData(err)),
    };

    if !res.status().is_success() {
        return Err(MyErrors::NasdaqNotOKResponse);
    }

    let body = match res.text().await {
        Ok(res) => res,
        Err(err) => return Err(MyErrors::CannotReadResponse(err)),
    };

    let mut records: Vec<Record> = Vec::new();

    for (key, line) in body.split("\n").into_iter().enumerate() {
        if let Ok(record) = parse_line(tick, key, line) {
            records.push(record);
        }
    }

    Ok(records)
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
