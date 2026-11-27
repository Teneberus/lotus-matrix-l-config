//! The content for [`crate::group::Group::local_line_data`],
//! [`crate::group::Group::local_line_e_data`] and for the local destination
//! content in the station strings of the special pis.

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;

/// This is the main struct for the display content.
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Content {
    /// Contains all content for the displays with their address as key.
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub displays: HashMap<usize, Display>,
}

impl Content {
    /// Parses TOML first, then JSON if TOML deserialization fails.
    pub fn from_str<'de, T: Deserialize<'de>>(s: &'de str) -> Result<T, crate::DeserializeError> {
        Self::from_slice(s.as_bytes())
    }

    /// Parses a byte slice as TOML first, then JSON.
    pub fn from_slice<'de, T: Deserialize<'de>>(
        s: &'de [u8],
    ) -> Result<T, crate::DeserializeError> {
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
}

/// Contains all content for the display.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Display {
    /// Display inversion. Is used together with [`Cycle::inverted`] as
    /// `display.inverted ^ cycle.inverted`.
    pub inverted: bool,
    /// List of cycles. Static content has exactly one cycle.
    /// With more than on cycle, the display loops through all cycles.
    pub cycles: Vec<Cycle>,
}

impl Display {
    /// Returns each cycle with its effective inversion, color, and elements.
    ///
    /// Inherited elements come before the cycle's own elements. A cycle with
    /// `release_locks` clears inherited elements before adding its own elements.
    pub fn expanded_cycles(&self) -> Vec<ExpandedCycle<'_>> {
        let mut expanded = Vec::with_capacity(self.cycles.len());
        let mut locked = Vec::new();

        for cycle in &self.cycles {
            if cycle.release_locks {
                locked.clear();
            }

            let mut elements = Vec::with_capacity(locked.len() + cycle.elements.len());
            elements.extend(locked.iter().copied());
            for element in &cycle.elements {
                elements.push(element);
                let position = match element {
                    Element::Text { position, .. }
                    | Element::Rectangle { position, .. }
                    | Element::Bitmap { position, .. } => position,
                };
                if position.locked {
                    locked.push(element);
                }
            }
            expanded.push(ExpandedCycle {
                inverted: self.inverted ^ cycle.inverted,
                color: cycle.color,
                elements,
            });
        }

        expanded
    }

    /// Create text content without inversion, color or anything else.
    /// Just a simple string.
    ///
    /// More strings can be added with [`Cycle::add_string`].
    pub fn from_string(string: &str, font_index: usize, x: i32, y: i32, spacing: i32) -> Self {
        Self {
            inverted: false,
            cycles: vec![Cycle {
                inverted: false,
                release_locks: false,
                color: None,
                elements: vec![Element::Text {
                    value: string.to_owned(),
                    font: font_index,
                    spacing,
                    position: Position {
                        h_align: HorizontalAlignment::None,
                        v_align: VerticalAlignment::None,
                        x,
                        y,
                        locked: false,
                    },
                    stroke: None,
                    color: None,
                    scroll_field: None,
                }],
            }],
        }
    }
}

/// One display cycle. The display loops through the cycles.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Cycle {
    /// Display inversion of this cycle. The effective inversion also depends on
    /// [`Display::inverted`].
    pub inverted: bool,
    /// Elements can be locked ([`Position::locked`]).
    /// In that case they are visible on all following cycles until this
    /// parameter here is set to true.
    /// Locked elements from previous cycles are not part of this cylce if this
    /// parameter is true.
    /// Elements, that are originally part of this cycle and are locked, are
    /// visible on this cycle and the following cycles.
    pub release_locks: bool,
    /// Background color of the RGB field.
    pub color: Option<Color>,
    /// List of elements that are added to this cycle. They can be locked
    /// ([`Position::locked`]), if they should be visible on following cycles
    /// too.
    pub elements: Vec<Element>,
}

impl Cycle {
    /// Adds a text element to the cycle.
    pub fn add_string(&mut self, string: &str, font_index: usize, x: i32, y: i32, spacing: i32) {
        self.elements.push(Element::Text {
            value: string.to_owned(),
            font: font_index,
            spacing,
            position: Position {
                h_align: HorizontalAlignment::None,
                v_align: VerticalAlignment::None,
                x,
                y,
                locked: false,
            },
            stroke: None,
            color: None,
            scroll_field: None,
        });
    }
}

/// Contains references to all elements of a cycle after processing the `locked`
/// and `release_locks` parameters (created with [`Display::expanded_cycles()`]).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpandedCycle<'a> {
    /// Effective inversion of the display.
    pub inverted: bool,
    /// Background color of the RGB field.
    pub color: Option<Color>,
    /// References to all elements that are visible on this cycle.
    pub elements: Vec<&'a Element>,
}

impl<'a> ExpandedCycle<'a> {
    /// Merges the cycles in lockstep until both sequences repeat.
    ///
    /// For each merged pair, the color from the first cycle replaces the
    /// color from the second cycle, even if the first color is `None`.
    ///
    /// If either sequence is empty, returns a copy of the other sequence.
    /// Merging two nonempty sequences returns an error at 50 or more cycles.
    pub fn merge(first: &[Self], second: &[Self]) -> Result<Vec<Self>, &'static str> {
        const CYCLE_LIMIT: usize = 50;

        if first.is_empty() {
            return Ok(second.to_vec());
        }
        if second.is_empty() {
            return Ok(first.to_vec());
        }

        let cycle_count = (1..CYCLE_LIMIT)
            .find(|&count| count % first.len() == 0 && count % second.len() == 0)
            .ok_or("Merged sequence would contain 50 or more cycles")?;

        let merged = first
            .iter()
            .cycle()
            .zip(second.iter().cycle())
            .take(cycle_count)
            .map(|(first, second)| Self {
                inverted: first.inverted ^ second.inverted,
                color: first.color,
                elements: first
                    .elements
                    .iter()
                    .chain(&second.elements)
                    .copied()
                    .collect(),
            })
            .collect();

        Ok(merged)
    }
}

/// Various elements with properties, that are positioned on the display.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Element {
    /// The text element writes a string with font, spacing, position, optional
    /// scroll values and some optional color parameters in case of RGB fields.
    #[serde(rename = "text")]
    Text {
        value: String,
        /// The font needs to exists in the font list in [`crate::group::Group`].
        #[serde(default)]
        font: usize,
        #[serde(default)]
        spacing: i32,
        /// Position (upper left corner).
        #[serde(default)]
        position: Position,
        /// Colored text stroke. Ignored, if scroll_field is Some.
        /// Only visible inside of RGB fields.
        stroke: Option<TextStroke>,
        /// Text color. Ignored, if scroll_field is Some.
        /// Only applied inside of RGB fields.
        color: Option<Color>,
        /// Scroll field with width, direction and speed.
        /// If this is not None, stroke and color will be ignored.
        scroll_field: Option<ScrollField>,
    },
    /// The rectangle element draws an empty rectangle with a given line width.
    #[serde(rename = "rectangle")]
    Rectangle {
        /// Outer width.
        #[serde(default)]
        width: u32,
        /// Outer height.
        #[serde(default)]
        height: u32,
        /// Border width.
        #[serde(default)]
        line_width: u32,
        /// Position (upper left corner).
        #[serde(default)]
        position: Position,
        /// Color. Only applied inside of RGB fields.
        color: Option<Color>,
    },
    /// The bitmap element places a bitmap at the given position.
    /// Only works in RGB fields.
    #[serde(rename = "bitmap")]
    Bitmap {
        /// Index of the bitmap.
        /// Accesses an ASCII char in the [`crate::group::Lotus::bitmap_font`]
        /// calculated as (`0x20 + index`).
        #[serde(default)]
        index: usize,
        /// Position (upper left corner).
        #[serde(default)]
        position: Position,
    },
}

/// Position of Elements.
///
/// The alignment parameters are placeholders for any graphical editors that
/// might be implemented later. They are not used by the displays.
///
/// Elements can be locked. In that case the are displayed on all following
/// cycles including the last one before the next cylce that releases all locks.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Position {
    pub h_align: HorizontalAlignment,
    pub v_align: VerticalAlignment,
    pub x: i32,
    pub y: i32,
    pub locked: bool,
}

impl Position {
    pub fn get_x_from_alignment(
        &self,
        display_width: u32,
        text_start: u32,
        text_width: u32,
    ) -> Result<i32, std::num::TryFromIntError> {
        let display_width = i64::from(display_width);
        let text_start = i64::from(text_start);
        let text_width = i64::from(text_width);

        let x = match &self.h_align {
            HorizontalAlignment::None => i64::from(self.x),
            HorizontalAlignment::LineLeft => 0,
            HorizontalAlignment::LineCenter => (text_start - text_width) / 2,
            HorizontalAlignment::LineRight => text_start - text_width,
            HorizontalAlignment::TextLeft => text_start,
            HorizontalAlignment::TextCenter => {
                let remaining_width = display_width - text_start - text_width;
                if remaining_width == 0 {
                    text_start
                } else {
                    text_start + (remaining_width / 2).max(1)
                }
            }
            HorizontalAlignment::TextRight => display_width - text_width,
            HorizontalAlignment::Center => (display_width - text_width) / 2,
        };

        i32::try_from(x)
    }

    pub fn get_y_from_alignment(
        &self,
        display_height: u32,
        text_height: u32,
    ) -> Result<i32, std::num::TryFromIntError> {
        let display_height = i64::from(display_height);
        let text_height = i64::from(text_height);

        let y = match &self.v_align {
            VerticalAlignment::None => i64::from(self.y),
            VerticalAlignment::Top => 0,
            VerticalAlignment::Middle => (display_height - text_height) / 2,
            VerticalAlignment::Bottom => display_height - text_height,
        };

        i32::try_from(y)
    }
}

/// Placeholder for any graphical editors that might be implemented later.
/// It is not used by the displays.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum HorizontalAlignment {
    #[default]
    None,
    LineLeft,
    LineCenter,
    LineRight,
    TextLeft,
    TextCenter,
    TextRight,
    Center,
}

impl std::fmt::Display for HorizontalAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HorizontalAlignment::None => write!(f, "None"),
            HorizontalAlignment::LineLeft => write!(f, "LineLeft"),
            HorizontalAlignment::LineCenter => write!(f, "LineCenter"),
            HorizontalAlignment::LineRight => write!(f, "LineRight"),
            HorizontalAlignment::TextLeft => write!(f, "TextLeft"),
            HorizontalAlignment::TextCenter => write!(f, "TextCenter"),
            HorizontalAlignment::TextRight => write!(f, "TextRight"),
            HorizontalAlignment::Center => write!(f, "Center"),
        }
    }
}

/// Placeholder for any graphical editors that might be implemented later.
/// It is not used by the displays.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum VerticalAlignment {
    #[default]
    None,
    Top,
    Middle,
    Bottom,
}

impl std::fmt::Display for VerticalAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerticalAlignment::None => write!(f, "None"),
            VerticalAlignment::Top => write!(f, "Top"),
            VerticalAlignment::Middle => write!(f, "Middle"),
            VerticalAlignment::Bottom => write!(f, "Bottom"),
        }
    }
}

/// Color for various elements with red, green and blue values.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Colored text stroke.
///
/// Currently I have no idea for a performance friendly implementation in LOTUS.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct TextStroke {
    pub width: u32,
    pub color: Color,
}

/// Width, direction and speed of scroll field
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ScrollField {
    /// Width of the scroll field.
    /// The height is taken from the text itself.
    pub width: u32,
    pub direction: Direction,
    /// Delay in centiseconds (cs) per column.
    ///
    /// Example: `delay = 2` is a delay of 20ms, which is a scroll speed of
    /// 50 columns per second.
    pub delay: u32,
}

/// Direction of scroll text.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Direction {
    #[default]
    RightToLeft,
    LeftToRight,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::RightToLeft => write!(f, "Right to left"),
            Direction::LeftToRight => write!(f, "Left to right"),
        }
    }
}
