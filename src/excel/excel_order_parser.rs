use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct ExcelOrderParser<'a> {
    pub path: &'a str,
    db: Pool<Postgres>,
}

impl<'a> ExcelOrderParser<'a> {
    pub fn new(path: &'a str, db: Pool<Postgres>) -> ExcelOrderParser<'a> {
        Self { path, db }
    }

    pub fn parse(&self) {}
}
