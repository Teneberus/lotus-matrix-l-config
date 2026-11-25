//! The group strings of the special pis containing general behavior, fonts, display settings and line/specialchar content.

pub mod display;

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::{collections::HashMap, ops::Range};

use crate::content::Content;

/// This is the main struct for the special pis group strings.
#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Group {
    /// LOTUS specific settings that are relevant for the simulation.
    pub lotus: Lotus,
    /// Settings, that are usually a firmware thing and cannot be configured in configuration files.
    pub system: System,
    /// General settings that apply to all displays.
    pub general: General,
    /// The fonts that are referenced from displays and local data.
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub font: HashMap<usize, Font>,
    /// Display configurations including size, fonts references and formatting options.
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub display: HashMap<usize, Display>,
    /// List of words that are replaced if the upper lower conversion is enabled in the display configuration.
    pub change_word_list: Option<Vec<(String, String)>>,
    /// List of line patterns with replacement strings.
    pub line_list: Option<Vec<(String, String)>>,
    /// List of specialchar codes and their replacement patterns for the line number.
    pub line_e_list: Option<Vec<(String, String)>>,
    /// Local line data that is only used with `lotus.local_data = true`.
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub local_line_content: Option<HashMap<usize, Content>>,
    /// Local specialchar data that is only used with `lotus.local_data = true`.
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub local_line_e_content: Option<HashMap<usize, Content>>,
}

impl Default for Group {
    fn default() -> Self {
        Self {
            lotus: Default::default(),
            system: Default::default(),
            general: Default::default(),
            font: HashMap::from([(0, Default::default())]),
            display: Default::default(),
            change_word_list: Default::default(),
            line_list: Default::default(),
            line_e_list: Default::default(),
            local_line_content: Default::default(),
            local_line_e_content: Default::default(),
        }
    }
}

impl Group {
    /// Wrapper for `toml::from_str()`.
    pub fn from_toml_str<'de, T>(s: &'de str) -> Result<T, toml::de::Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        toml::from_str(s)
    }

    /// Wrapper for `toml::from_slice()`.
    pub fn from_toml_slice<'de, T>(s: &'de [u8]) -> Result<T, toml::de::Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        toml::from_slice(s)
    }

    /// Returns the destination shift to the left for local destination data.
    ///
    /// If the line number is not empty and the width is smaller than the
    /// default width calculated from [`General::default_line_digit_count`],
    /// the shift is the rounded down half of the width difference.
    /// Otherwise, the shift is 0.
    ///
    /// The shift is also 0, if one of [`System::destination_shift`] or
    /// [`Lotus::local_data`] is false, a nonexistent display address is given
    /// or a nonexistent font is defined in the display configuration.
    pub fn get_destination_shift(
        &self,
        line_width_px: u32,
        line_char_count: usize,
        display_address: usize,
    ) -> u32 {
        if !self.system.destination_shift || !self.lotus.local_data || (line_char_count == 0) {
            return 0;
        }

        let Some(display) = self.display.get(&display_address) else {
            return 0;
        };

        let Some(font) = self.font.get(&display.line.font) else {
            return 0;
        };

        let default_width = self.general.default_line_digit_count * font.zero_width
            + (self.general.default_line_digit_count - 1)
                * display.line.spacing[line_char_count.saturating_sub(1) & 3]
            + display.line.offset
                [(self.general.default_line_digit_count as usize).saturating_sub(1) & 3];

        default_width
            .saturating_sub(line_width_px)
            .saturating_sub(1)
            / 2
    }
}

/// LOTUS specific settings that are relevant for the simulation.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Lotus {
    /// This enables local data. With this set to true, the station strings of
    /// the special pis are parsed for preconfigured destination content and
    /// [`Group::local_line_content`] and [`Group::local_line_e_content`] are
    /// processed.
    /// Default: false.
    pub local_data: bool,
    /// For usability the display configuration and the destination contents are
    /// searched using the display size. Setting this parameter to true, will
    /// match this data based on the address, which is more realistic, but also
    /// makes the configurations more vehicle specific.
    ///
    /// Be aware, that displays in LOTUS, that find no configuration or data,
    /// will stay empty.
    ///
    /// Default: false.
    pub match_display_address: bool,
    /// If set to true, all texts read from pis (and special pis) that are not
    /// preconfigured, are converted to IBIS conmpliant uppercase strings.
    ///
    /// If set to false, those texts are only converted to IBIS compliant
    /// strings without the uppercase part.
    ///
    /// The conversion to IBIS compliant strings means, the all non-ASCII chars
    /// are thrown away except `Ä`, `Ö`, `Ü`, `ä`, `ö`, `ü`, `ß`, `🚇` and `🚈`.
    /// Those are converted into `[`, `\`, `]`, `{`, `|`, `}`, `~`, `^` and `_`.
    ///
    /// Default: false.
    pub set_uppercase_input: bool,
    /// Displays with RGB fields can implement graphics. This defines the
    /// LOTUS-Font containing those images.
    pub bitmap_font: Option<Font>,
}

/// Settings, that are usually a firmware thing and cannot be configured in configuration files.
#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct System {
    /// Enables the destination shift of preconfigured destinations if the line
    /// number is smaller than the default width.
    /// Default: false.
    pub destination_shift: bool,
    /// With `lotus.local_data = true` this enables the fallback to default pis
    /// (and special pis) texts including upper lower text conversion, if no
    /// local content is found for the received destination.
    /// Default: false.
    pub allow_fallback: bool,
    /// With `lotus.local_data = true` this excludes a destination code range.
    /// If a code in this range is received, the dynamic mode is used as if
    /// `lotus.local_data` is false.
    pub dynamic_range: Option<Range<usize>>,
    /// Sets the line to 0, if [`Lotus::local_data`] is true except in the
    /// dynamic range.
    pub no_line_with_local_data: bool,
    /// Internal settings for the upper lower conversion algorithm
    /// ([`crate::ToUpperLowerCase`], [`crate::ToUpperLowerCasePair`]).
    pub upper_lower_config: crate::ToUpperLowerConfig,
    /// Strings are checked for these chars to determine the vertical alignment.
    ///
    /// Default: `<`, `=`, `>`, `g`, `j`, `p`, `q` and `y`.
    pub chars_with_descender: Vec<char>,
    /// Optional boot content. This content is shown on display start before
    /// transition into normal operation.
    pub startup_content: Option<SystemContent>,
    /// Optional specialchar overrides. For example, specialchar 99 can show
    /// system information with this.
    /// Specialchars in this have a higher priority than other specialchar
    /// definitions.
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub line_e_content: Option<HashMap<usize, SystemContent>>,
    /// Enables the vehicle number if line and specialchar are 0.
    /// Default: false.
    pub show_vehicle_number: bool,
}

impl Default for System {
    fn default() -> Self {
        Self {
            destination_shift: false,
            allow_fallback: false,
            dynamic_range: None,
            no_line_with_local_data: false,
            upper_lower_config: Default::default(),
            chars_with_descender: vec!['<', '=', '>', 'g', 'j', 'p', 'q', 'y'],
            startup_content: Default::default(),
            line_e_content: Default::default(),
            show_vehicle_number: false,
        }
    }
}

/// System content wrapper with individual cycle times.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct SystemContent {
    /// [`General::cycle_time`] is replaced by these times.
    /// It is not required to set enough cycle times for all cycles.
    /// Missing values will use [`General::cycle_time`].
    pub cycle_times: Vec<f32>,
    pub content: Content,
}

/// General settings that apply to all displays.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct General {
    /// A comment for the log to verify the correct config is loaded.
    pub comment: String,
    /// Cycle time for destinations with multiple cycles in seconds.
    /// Default: 3.0
    pub cycle_time: f32,
    /// Maximum allowed spacing for automatic formatting of destinations fonts.
    /// Default: 3
    pub max_spacing: u32,
    /// Timeout in seconds after which the display is cleared, if no data is
    /// received. 0.0 disables the timeout.
    /// Default: 120.0
    pub ibis_timeout: f32,
    /// If true, decode `[`, `\`, `]`, `{`, `|`, and `}` as `Ä`, `Ö`, `Ü`, `ä`, `ö`, and
    /// `ü`, respectively, and decode `^` and `~` as `ß`.
    ///
    /// This should be used in [`crate::ToUpperLowerCase`] and
    /// [`crate::ToUpperLowerCasePair`].
    ///
    /// Default: true
    pub convert_chars: bool,
    /// This is used to calculate the default width of a line number.
    /// Default: 3
    pub default_line_digit_count: u32,
}

impl Default for General {
    fn default() -> Self {
        Self {
            comment: "".to_string(),
            cycle_time: 3.0,
            max_spacing: 3,
            ibis_timeout: 120.0,
            convert_chars: true,
            default_line_digit_count: 3,
        }
    }
}

/// The fonts that are referenced from displays and local data.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Font {
    pub user_id: i32,
    pub sub_id: i32,
    pub height: u32,
    /// The baseline is required for vertical text alignment.
    #[serde(default)]
    pub baseline: u32,
    /// The width of the digit `0`. This is required for line number fonts to
    /// calculate the default line number width together with
    /// [`General::default_line_digit_count`].
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

/// Display configurations including size, fonts references and formatting options.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Display {
    /// Size of display fields for line numbers and text.
    pub size: display::Size,
    /// Size and offset of the RGB area on the display.
    #[serde(default)]
    pub color: display::Color,
    /// Line number formatting settings.
    #[serde(default)]
    pub line: display::Line,
    /// Destination text formatting settings.
    #[serde(default)]
    pub destination: display::Destination,
    /// Minimum spacing between line number and destination text.
    #[serde(default = "get_display_line_destination_spacing")]
    pub line_destination_spacing: u32,
    /// Display orientation (Front or Side).
    #[serde(default)]
    pub orientation: display::Orientation,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            size: Default::default(),
            color: Default::default(),
            line: Default::default(),
            destination: Default::default(),
            line_destination_spacing: get_display_line_destination_spacing(),
            orientation: Default::default(),
        }
    }
}

fn get_display_line_destination_spacing() -> u32 {
    3
}
