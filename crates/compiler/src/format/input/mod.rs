use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
pub struct Bundle {
    pub badge: Badge,
    pub conditions: Conditions,
    pub lang: HashMap<String, Locale>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Badge {
    pub group: Option<String>,
    #[serde(default)]
    pub points: u16,
    pub map: Map,
    pub art: String,
    #[serde(default)]
    pub animated: bool,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Map {
    Plain(u16),
    Object {
        id: u16,
        #[serde(default)]
        x: u16,
        #[serde(default)]
        y: u16,
        #[serde(default)]
        secret: bool,
    },
}

#[derive(Debug, serde::Deserialize)]
pub struct Conditions {
    #[serde(default)]
    pub secret: bool,
    pub default: Option<String>,
    pub requirements: Option<String>,
    #[serde(flatten)]
    pub rest: HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Locale {
    pub name: Option<String>,
    pub description: Option<String>,
    pub condition: Option<String>,
    pub checkbox: Option<HashMap<String, String>>,
}
