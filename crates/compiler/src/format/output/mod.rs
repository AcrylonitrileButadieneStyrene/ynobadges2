#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub animated: bool,
    pub art: String,
    pub batch: u16,
    pub bp: u16,
    pub group: String,
    pub hidden: bool,
    pub map: u16,
    pub map_order: u8,
    pub map_x: u16,
    pub map_y: u16,
    pub order: u8,
    pub overlay_type: u8,
    pub parent: String,
    pub req_count: u8,
    pub req_int: u16,
    pub req_string: String,
    pub req_string_arrays: Vec<Vec<String>>,
    pub req_strings: Vec<String>,
    pub req_type: BadgeReqType,
    pub secret: bool,
    pub secret_condition: bool,
    pub secret_map: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum BadgeReqType {
    BadgeCount,
    Exp,
    ExpCompletion,
    ExpCount,
    LocationCompletion,
    Medal,
    Tag,
    TagArrays,
    Tags,
    TimeTrial,
    VmCount,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub map: u16,
    pub map_x1: u16,
    pub map_x2: u16,
    pub map_y1: u16,
    pub map_y2: u16,
    pub switch_delay: bool,
    pub switch_id: u16,
    pub switch_ids: Vec<u16>,
    pub switch_value: bool,
    pub switch_values: Vec<bool>,
    pub time_trial: bool,
    pub trigger: ConditionTrigger,
    pub value: String,
    pub values: Vec<String>,
    pub var_delay: bool,
    pub var_id: u16,
    pub var_ids: Vec<u16>,
    pub var_op: String,
    pub var_ops: Vec<String>,
    pub var_trigger: bool,
    pub var_value: u32,
    pub var_value2: u32,
    pub var_values: Vec<u32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum ConditionTrigger {
    Event,
    EventAction,
    Coords,
    Teleport,
    Picture,
    PrevMap,
}

pub type Lang = std::collections::HashMap<
    String, // game
    std::collections::HashMap<
        String, // req string
        super::input::Locale,
    >,
>;
