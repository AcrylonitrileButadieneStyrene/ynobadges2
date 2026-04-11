use std::collections::HashMap;

#[derive(serde::Deserialize)]
pub struct Config {
    pub lang: Lang,
    pub groups: HashMap<String, Group>,
}

#[derive(serde::Deserialize)]
pub struct Lang {
    pub base: String,
    pub list: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct Group {
    pub default: Option<String>,
    pub list: Vec<String>,
}
