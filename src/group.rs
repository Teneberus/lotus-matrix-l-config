use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::content::Content;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Group {
    pub lotus: Lotus,
    pub system: System,
    pub general: General,
    pub font: HashMap<String, Font>,
    pub display: HashMap<String, Display>,
    pub change_word_list: Option<Vec<(String, String)>>,
    pub line_list: Option<Vec<(String, String)>>,
    pub special_char_list: Option<Vec<(String, String)>>,
    pub startup_content: Option<Content>,
    pub line_content: Option<HashMap<String, Content>>,
    pub line_e_content: Option<HashMap<String, Content>>,
}

impl Default for Group {
    fn default() -> Self {
        Self {
            lotus: Default::default(),
            system: Default::default(),
            general: Default::default(),
            font: HashMap::from([("0".to_string(), Default::default())]),
            display: Default::default(),
            change_word_list: Default::default(),
            line_list: Default::default(),
            special_char_list: Default::default(),
            startup_content: Default::default(),
            line_content: Default::default(),
            line_e_content: Default::default(),
        }
    }
}

impl Group {
    pub fn from_toml_str<'de, T>(s: &'de str) -> Result<T, toml::de::Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        toml::from_str(s)
    }

    pub fn from_toml_slice<'de, T>(s: &'de [u8]) -> Result<T, toml::de::Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        toml::from_slice(s)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Lotus {
    pub use_special_char_list: bool,
    pub match_display_address: bool,
    pub bitmap_font: Option<Font>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct System {
    pub datetimes: Datetimes,
    pub local_data: bool,
    pub destination_shift: bool,
    pub line_fallback: bool,
    pub destination_fallback: bool,
    pub upper_lower_config: crate::ToUpperLowerConfig,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Datetimes {
    pub bic: toml::value::Datetime,
    pub lop: toml::value::Datetime,
    pub mo1: toml::value::Datetime,
}

impl Default for Datetimes {
    fn default() -> Self {
        Self {
            bic: toml::value::Datetime::from_str("1970-01-01T00:00:00Z").unwrap(),
            lop: toml::value::Datetime::from_str("1970-01-01T00:00:00Z").unwrap(),
            mo1: toml::value::Datetime::from_str("1970-01-01T00:00:00Z").unwrap(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct General {
    pub comment: String,
    pub cycle_time: f32,
    pub max_spacing: u32,
    pub ibis_timeout: f32,
    pub convert_chars: bool,
}

impl Default for General {
    fn default() -> Self {
        Self {
            comment: "Cfg-LOTUS".to_string(),
            cycle_time: 3.0,
            max_spacing: 3,
            ibis_timeout: 120.0,
            convert_chars: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Font {
    pub user_id: u32,
    pub sub_id: u32,
    pub height: u32,
    #[serde(default)]
    pub baseline: u32,
    #[serde(default)]
    pub zero_width: u32,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            user_id: 5747312,
            sub_id: 457430500,
            height: 8,
            baseline: 7,
            zero_width: 4,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Display {
    pub size: DisplaySize,
    #[serde(default)]
    pub color: DisplayColor,
    #[serde(default)]
    pub line: DisplayLine,
    #[serde(default)]
    pub destination: DisplayDestination,
    #[serde(default = "get_display_line_destination_spacing")]
    pub line_destination_spacing: u32,
    #[serde(default)]
    pub upper_lower_convert: UpperLowerConvert,
    #[serde(default)]
    pub orientation: DisplayOrientation,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            size: Default::default(),
            color: Default::default(),
            line: Default::default(),
            destination: Default::default(),
            line_destination_spacing: get_display_line_destination_spacing(),
            upper_lower_convert: Default::default(),
            orientation: Default::default(),
        }
    }
}

fn get_display_line_destination_spacing() -> u32 {
    3
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DisplaySize {
    pub width: u32,
    pub height: u32,
    pub text_start: u32,
    #[serde(default)]
    pub line_width: u32,
    #[serde(default)]
    pub line_height: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DisplayColor {
    pub offset: u32,
    pub width: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DisplayLine {
    pub suppress: bool,
    pub font: usize,
    pub place: LinePlace,
    pub offset: [u32; 4],
    pub spacing: [u32; 4],
}

impl Default for DisplayLine {
    fn default() -> Self {
        Self {
            suppress: false,
            font: 0,
            place: Default::default(),
            offset: [3, 2, 1, 1],
            spacing: [0, 2, 1, 1],
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DisplayDestination {
    pub font: DestinationFont,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct UpperLowerConvert {
    pub one_line: bool,
    pub two_lines: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum DisplayOrientation {
    #[default]
    Front,
    Side,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum LinePlace {
    Line,
    #[default]
    Text,
    Dynamic,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DestinationFont {
    pub one_line: Vec<usize>,
    pub two_lines_up: Vec<usize>,
    pub two_lines_lo: Vec<usize>,
}

impl Default for DestinationFont {
    fn default() -> Self {
        Self {
            one_line: vec![0],
            two_lines_up: vec![0],
            two_lines_lo: vec![0],
        }
    }
}
