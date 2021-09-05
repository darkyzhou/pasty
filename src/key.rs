pub fn type_key(id: &str) -> String {
    format!("{}_type", id)
}

pub fn content_key(id: &str) -> String {
    format!("{}_content", id)
}

pub fn password_key(id: &str) -> String {
    format!("{}_password", id)
}

pub fn stat_count_key(id: &str) -> String {
    format!("{}_stat_count", id)
}
