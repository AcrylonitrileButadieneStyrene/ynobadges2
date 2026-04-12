fn default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
}

#[serde_with::skip_serializing_none]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    #[serde(skip_serializing_if = "default")]
    pub animated: bool,
    pub art: String,
    pub batch: u16,
    // todo: change this to NonZeroU16 when `just_smile_dog_aseprite` gets fixed
    pub bp: Option<u16>,
    pub group: Option<String>,
    #[serde(skip_serializing_if = "default")]
    pub hidden: bool,
    pub map: u16,
    pub map_order: Option<u8>,
    pub map_x: Option<u16>,
    pub map_y: Option<u16>,
    pub order: Option<u8>,
    pub overlay_type: Option<u8>,
    pub parent: Option<String>,
    pub req_count: Option<u8>,
    pub req_int: Option<u16>,
    pub req_string: Option<String>,
    pub req_string_arrays: Option<Vec<Vec<String>>>,
    pub req_strings: Option<Vec<String>>,
    pub req_type: Option<BadgeReqType>,
    #[serde(skip_serializing_if = "default")]
    pub secret: bool,
    #[serde(skip_serializing_if = "default")]
    pub secret_condition: bool,
    #[serde(skip_serializing_if = "default")]
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

#[serde_with::skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub map: Option<u16>,
    pub map_x1: Option<u16>,
    pub map_x2: Option<u16>,
    pub map_y1: Option<u16>,
    pub map_y2: Option<u16>,
    #[serde(skip_serializing_if = "default")]
    pub switch_delay: bool,
    pub switch_id: Option<u16>,
    pub switch_ids: Option<Vec<u16>>,
    // this is rarely omitted
    #[serde(skip_serializing_if = "default")]
    pub switch_value: bool,
    pub switch_values: Option<Vec<bool>>,
    pub time_trial: bool,
    pub trigger: Option<ConditionTrigger>,
    pub value: Option<String>,
    pub values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "default")]
    pub var_delay: bool,
    pub var_id: Option<u16>,
    pub var_ids: Option<Vec<u16>>,
    pub var_op: Option<String>,
    pub var_ops: Option<Vec<String>>,
    #[serde(skip_serializing_if = "default")]
    pub var_trigger: bool,
    pub var_value: Option<i32>,
    pub var_value2: Option<i32>,
    pub var_values: Option<Vec<i32>>,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum ConditionTrigger {
    Event,
    EventAction,
    Coords,
    Teleport,
    Picture,
    PrevMap,
}

pub use super::input::Locale;

pub type Lang = std::collections::HashMap<
    String, // game
    std::collections::HashMap<
        String, // req string
        Locale,
    >,
>;
