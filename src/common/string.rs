

pub fn common_prefix(strs: Vec<&str>)-> String {

    "".to_string()
}

pub fn remove_whitespace_string(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

pub fn remove_whitespace_str(s: &str)-> String {
    s.chars().filter(|c|!c.is_whitespace()).collect()
}