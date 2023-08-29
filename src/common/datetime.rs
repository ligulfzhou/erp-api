use chrono::NaiveDate;

pub fn parse_date(date: &str) -> Option<NaiveDate> {
    match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(res) => Some(res),
        Err(_) => match NaiveDate::parse_from_str(date, "%Y/%m/%d") {
            Ok(res) => Some(res),
            Err(_) => None,
        },
    }
}
