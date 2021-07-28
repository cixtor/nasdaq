mod accounts;

use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
pub enum MyErrors {
    CannotOpenFile(std::io::Error),
    CannotReadFileInReverse(std::io::Error),
    FileHasNoLines,
}

fn main() {
    for (tick, filename) in accounts::get_accounts().iter() {
        println!("{} {}", tick, filename);

        if let Err(err) = read_last_date_from_file(filename) {
            println!("{:?}", err);
        }
    }
}

fn read_last_date_from_file(filename: &str) -> Result<(), MyErrors> {
    let file = match File::open(filename) {
        Ok(res) => res,
        Err(err) => return Err(MyErrors::CannotOpenFile(err)),
    };

    let mut rev_lines = match rev_lines::RevLines::new(BufReader::new(file)) {
        Ok(res) => res,
        Err(err) => return Err(MyErrors::CannotReadFileInReverse(err)),
    };

    let last_line = match rev_lines.next() {
        Some(res) => res,
        None => return Err(MyErrors::FileHasNoLines),
    };

    println!("{}", last_line);

    Ok(())
}
