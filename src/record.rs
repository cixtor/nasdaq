use core::fmt;

pub struct Record {
    pub date: String,
    pub payee: String,
    pub category: String,
    pub note: String,
    pub amount: f64,
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{:.2}",
            self.date, self.payee, self.category, self.note, self.amount,
        )
    }
}
