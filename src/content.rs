use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::{cmp::max, collections::HashMap, fmt::Display};

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Content {
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub address: HashMap<usize, Address>,
}

impl Content {
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Address {
    pub inverted: bool,
    pub cycles: Vec<Cycle>,
}

impl Address {
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
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Cycle {
    pub inverted: bool,
    pub release_locks: bool,
    pub color: Option<Color>,
    pub elements: Vec<Element>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpandedCycle<'a> {
    pub inverted: bool,
    pub color: Option<Color>,
    pub elements: Vec<&'a Element>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Element {
    #[serde(rename = "text")]
    Text {
        value: String,
        #[serde(default)]
        font_index: usize,
        #[serde(default)]
        spacing: i32,
        #[serde(default)]
        position: Position,
        frame: Option<TextFrame>,
        color: Option<Color>,
        scroll_field: Option<ScrollField>,
    },
    #[serde(rename = "rectangle")]
    Rectangle {
        #[serde(default)]
        width: u32,
        #[serde(default)]
        height: u32,
        #[serde(default)]
        line_width: u32,
        #[serde(default)]
        position: Position,
        color: Option<Color>,
    },
    #[serde(rename = "bitmap")]
    Bitmap {
        #[serde(default)]
        index: usize,
        #[serde(default)]
        position: Position,
    },
}

// common

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
                    text_start + max(1, remaining_width / 2)
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

impl Display for HorizontalAlignment {
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum VerticalAlignment {
    #[default]
    None,
    Top,
    Middle,
    Bottom,
}

impl Display for VerticalAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerticalAlignment::None => write!(f, "None"),
            VerticalAlignment::Top => write!(f, "Top"),
            VerticalAlignment::Middle => write!(f, "Middle"),
            VerticalAlignment::Bottom => write!(f, "Bottom"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct TextFrame {
    pub width: u32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ScrollField {
    pub width: u32,
    pub height: u32,
    pub direction: Direction,
    pub delay: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Direction {
    #[default]
    RightToLeft,
    LeftToRight,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::RightToLeft => write!(f, "Right to left"),
            Direction::LeftToRight => write!(f, "Left to right"),
        }
    }
}
