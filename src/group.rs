//! The group strings of the special pis containing general behavior, fonts, display settings and line/specialchar content.

pub mod display;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::{collections::HashMap, ops::RangeInclusive};

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
    pub fonts: HashMap<usize, Font>,
    /// Display configurations including size, fonts references and formatting options.
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub displays: HashMap<usize, Display>,
    /// List of words that are replaced if the upper lower conversion is enabled in the display configuration.
    pub change_word_list: Option<Vec<(String, String)>>,
    /// List of line patterns with replacement strings.
    pub line_list: Option<Vec<(String, String)>>,
    /// List of specialchar codes and their replacement patterns for the line number.
    pub line_e_list: Option<Vec<(String, String)>>,
    /// Local line data that is only used with `lotus.local_data = true`.
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub local_line_data: Option<HashMap<usize, Content>>,
    /// Local specialchar data that is only used with `lotus.local_data = true`.
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub local_line_e_data: Option<HashMap<usize, Content>>,
}

impl Default for Group {
    fn default() -> Self {
        Self {
            lotus: Default::default(),
            system: Default::default(),
            general: Default::default(),
            fonts: HashMap::from([(0, Default::default())]),
            displays: Default::default(),
            change_word_list: Default::default(),
            line_list: Default::default(),
            line_e_list: Default::default(),
            local_line_data: Default::default(),
            local_line_e_data: Default::default(),
        }
    }
}

impl Group {
    /// Parses TOML first, then JSON if TOML deserialization fails.
    pub fn from_str(s: &str) -> Result<Self, crate::DeserializeError> {
        Self::from_slice(s.as_bytes())
    }

    /// Parses a byte slice as TOML first, then JSON.
    pub fn from_slice(s: &[u8]) -> Result<Self, crate::DeserializeError> {
        crate::deserialize_config(s)
    }

    /// Serializes the configuration as TOML.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    /// Serializes the configuration as JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Formats the line using the first matching pattern in [`Self::line_list`].
    ///
    /// Returns `None` if the list is absent or no pattern matches.
    pub fn format_line(&self, line: u32, specialchar: u32) -> Option<String> {
        let list = self.line_list.as_deref()?;
        let line_key = line.to_string();
        let template = first_line_template(list, &line_key)?;
        let line_digits = if line == 0 { "" } else { &line_key };
        render_line_template(template, line_digits, &specialchar.to_string())
    }

    /// Formats the line using the first matching specialchar in [`Self::line_e_list`].
    ///
    /// Returns `None` for specialchar zero, an absent list, or a missing match.
    pub fn format_line_e(&self, line: u32, specialchar: u32) -> Option<String> {
        let list = self.line_e_list.as_deref()?;
        if specialchar == 0 {
            return None;
        }

        let specialchar_key = specialchar.to_string();
        let template = first_line_template(list, &specialchar_key)?;
        let line_digits = if line == 0 {
            String::new()
        } else {
            line.to_string()
        };
        render_line_template(template, &line_digits, &specialchar_key)
    }

    /// Applies the system's line zero trimming options around the first
    /// character that is not an ASCII digit.
    ///
    /// Leading trimming removes zeros immediately after that character.
    /// Trailing trimming removes zeros immediately before it.
    /// A string made entirely of digits is returned as-is.
    pub fn trim_line_e_zeros(&self, text: &str) -> String {
        let Some((index, marker)) = text.char_indices().find(|(_, ch)| !ch.is_ascii_digit()) else {
            return text.to_owned();
        };
        let (before, rest) = text.split_at(index);
        let (marker, after) = rest.split_at(marker.len_utf8());

        let before = if self.system.cut_trailing_line_e_zeros {
            before.trim_end_matches('0')
        } else {
            before
        };
        let after = if self.system.cut_leading_line_e_zeros {
            after.trim_start_matches('0')
        } else {
            after
        };

        [before, marker, after].concat()
    }

    /// Uses the configured matching mode to find the display address needed
    /// to access the display's configuration and contents.
    ///
    /// When `lotus.match_display_address` is enabled, returns `Some(address)`
    /// if the address exists or `None` otherwise.
    ///
    /// When `lotus.match_display_address` is disabled, selects among displays
    /// with the given `width` and `height`, ordered by ascending address.
    /// The `index` is zero-based. If it exceeds the last valid index, returns
    /// the highest matching address. If no display matches, returns `None`.
    pub fn find_display_address(
        &self,
        address: usize,
        width: u32,
        height: u32,
        index: usize,
    ) -> Option<usize> {
        if self.lotus.match_display_address {
            if self.displays.contains_key(&address) {
                Some(address)
            } else {
                None
            }
        } else {
            let mut matching_addresses: Vec<usize> = self
                .displays
                .iter()
                .filter_map(|(&address, display)| {
                    (display.size.width == width && display.size.height == height)
                        .then_some(address)
                })
                .collect();

            matching_addresses.sort_unstable();

            let index = index.min(matching_addresses.len().saturating_sub(1));
            matching_addresses.get(index).copied()
        }
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

        let Some(display) = self.displays.get(&display_address) else {
            return 0;
        };

        let Some(font) = self.fonts.get(&display.line.font) else {
            return 0;
        };

        let default_width = self.general.default_line_digit_count * font.digit_width
            + (self.general.default_line_digit_count - 1)
                * display.line.spacings[line_char_count.saturating_sub(1) & 3]
            + display.line.offsets
                [(self.general.default_line_digit_count as usize).saturating_sub(1) & 3];

        default_width
            .saturating_sub(line_width_px)
            .saturating_sub(1)
            / 2
    }
}

/// Finds the first matching pattern in the (pattern, template) list and returns the
/// found template or None if none is found.
/// Returns the template from the first pattern that matches the key.
///
/// Patterns must have the same length as the key. Each `x` or `X`
/// matches one digit, and all other characters must match literally.
fn first_line_template<'a>(list: &'a [(String, String)], key: &str) -> Option<&'a str> {
    list.iter()
        .find(|(pattern, _)| {
            pattern.len() == key.len()
                && pattern
                    .bytes()
                    .zip(key.bytes())
                    .all(|(pattern, digit)| matches!(pattern, b'x' | b'X') || pattern == digit)
        })
        .map(|(_, template)| template.as_str())
}

/// Renders line and specialchar digits into a replacement template.
///
/// A digit after `#` selects a line digit, while a digit after `*`
/// selects a specialchar digit. Index zero is the rightmost digit.
/// Missing digits are skipped. A marker followed by another marker
/// outputs the second marker literally.
fn render_line_template(template: &str, line: &str, specialchar: &str) -> Option<String> {
    let mut text = String::new();
    let mut chars = template.chars();

    while let Some(ch) = chars.next() {
        if !matches!(ch, '#' | '*') {
            text.push(ch);
            continue;
        }

        let Some(next) = chars.next() else {
            break;
        };

        match next {
            '#' | '*' => text.push(next),
            '0'..='9' => {
                let digits = if ch == '#' { line } else { specialchar };
                let index = (next as u8 - b'0') as usize;
                if let Some(digit) = digits.bytes().rev().nth(index) {
                    text.push(char::from(digit));
                }
            }
            _ => {}
        }
    }

    Some(text)
}

/// LOTUS specific settings that are relevant for the simulation.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Lotus {
    /// This enables local data. With this set to true, the station strings of
    /// the special pis are parsed for preconfigured destination content and
    /// [`Group::local_line_data`] and [`Group::local_line_e_data`] are
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
    /// With `lotus.local_data = true` this excludes destination code ranges.
    /// If a code in one of these ranges is received, the dynamic mode is used
    /// as if `lotus.local_data` is false.
    /// Deserialization sorts the ranges and merges overlapping or adjacent ranges.
    #[serde(deserialize_with = "deserialize_dynamic_destination_ranges")]
    pub dynamic_destination_ranges: Vec<RangeInclusive<usize>>,
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
    pub startup: Option<SystemContent>,
    /// Optional specialchar overrides. For example, specialchar 99 can show
    /// system information with this.
    /// Specialchars in this have a higher priority than other specialchar
    /// definitions.
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub line_e_overrides: Option<HashMap<usize, SystemContent>>,
    /// Limit incoming line number length.
    /// Default: 4.
    pub max_line_digits: u32,
    /// Leading zeros after non-digits chars are cut (M01 -> M1).
    /// Default: false.
    pub cut_leading_line_e_zeros: bool,
    /// Trailing zeros before non-digit chars are cut (50N -> 5N).
    /// Default: false.
    pub cut_trailing_line_e_zeros: bool,
    /// Enables the vehicle number if line and specialchar are 0.
    /// Default: false.
    pub show_vehicle_number: bool,
}

impl Default for System {
    fn default() -> Self {
        Self {
            destination_shift: false,
            allow_fallback: false,
            dynamic_destination_ranges: Default::default(),
            no_line_with_local_data: false,
            upper_lower_config: Default::default(),
            chars_with_descender: vec!['<', '=', '>', 'g', 'j', 'p', 'q', 'y'],
            startup: Default::default(),
            line_e_overrides: Default::default(),
            max_line_digits: 4,
            cut_leading_line_e_zeros: false,
            cut_trailing_line_e_zeros: false,
            show_vehicle_number: false,
        }
    }
}

impl System {
    /// Returns whether the destination code is in any configured dynamic range.
    ///
    /// The ranges must be sorted and nonoverlapping, as produced by deserialization.
    pub fn dynamic_ranges_contain(&self, destination: usize) -> bool {
        let ranges = &self.dynamic_destination_ranges;
        let index = ranges.partition_point(|range| *range.start() <= destination);
        index > 0 && ranges[index - 1].contains(&destination)
    }
}

fn deserialize_dynamic_destination_ranges<'de, D>(
    deserializer: D,
) -> Result<Vec<RangeInclusive<usize>>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut ranges = Vec::<RangeInclusive<usize>>::deserialize(deserializer)?;
    ranges.retain(|range| !range.is_empty());
    ranges.sort_unstable_by_key(|range| *range.start());

    let mut merged: Vec<RangeInclusive<usize>> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut() {
            if *range.start() <= last.end().saturating_add(1) {
                *last = *last.start()..=(*last.end()).max(*range.end());
                continue;
            }
        }
        merged.push(range);
    }
    Ok(merged)
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
    pub digit_width: u32,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            user_id: 5747312,
            sub_id: 457430500,
            height: 8,
            baseline: 7,
            digit_width: 4,
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
    #[serde(default = "default_line_destination_spacing")]
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
            line_destination_spacing: default_line_destination_spacing(),
            orientation: Default::default(),
        }
    }
}

fn default_line_destination_spacing() -> u32 {
    3
}
