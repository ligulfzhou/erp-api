use std::collections::HashMap;

pub const DEFAULT_PAGE_SIZE: i32 = 50;

pub const STORAGE_FILE_PATH: &str = "/home/debian/data/file/";
// pub const STORAGE_FILE_PATH: &str = "/Users/ligangzhou/data/file/";

pub const STORAGE_URL_PREFIX: &str = "https://erp.ligulfzhou.com/file/";
// pub const STORAGE_URL_PREFIX: &str = "http://localhost:9100/file/";

lazy_static! {
    pub static ref STEP_TO_DEPARTMENT: HashMap<i32, &'static str> =
        vec![(1, "业务部"), (2, "仓库"), (3, "车间"),]
            .into_iter()
            .collect();
}
