//! Size, color, line number and destination options for displays.

use serde::{Deserialize, Serialize};

/// Size of display fields for line numbers and text.
///
/// The size of the text field is set by `width` and `height`.
/// The size of the line field is set by `line_width` and `line_height`.
///
/// Most displays usually just have a text field containing the
/// line number and the destination text (with `line.place` set to `Text` or
/// `Dynamic`, more on that in [`Line`], [`LinePlace`]).
///
/// The line field should be used if the line number is on a separated display
/// module.
///
/// `text_start` is a placeholder for any graphical editors that might be
/// implemented later. It is not used by the displays.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Size {
    /// Width of the text field.
    pub width: u32,
    /// Height of the text field.
    pub height: u32,
    /// Width of the line field.
    #[serde(default)]
    pub line_width: u32,
    /// Height of the line field.
    #[serde(default)]
    pub line_height: u32,
    /// Placeholder. Not used by displays.
    #[serde(default)]
    pub text_start: u32,
}

/// Size and offset of the RGB area on the display.
///
/// Usually the offset is 0 and the width is the width of the first module.
/// The height of the RGB area always matches the display/module height.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Color {
    /// Offset of the RGB area.
    pub offset: u32,
    /// Width of the RGB area.
    pub width: u32,
}

/// Line number formatting settings.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Line {
    /// Suppress the line number on the display. Only the destination text is
    /// displayed.
    pub suppress: bool,
    /// Line number font. The font needs to exists in the font list in
    /// [`crate::group::Group`].
    pub font: usize,
    /// Optional alternative font in case the line number does not fit with
    /// `font`.
    /// Only if [`super::Lotus::local_data`] is false or
    /// [`super::System::allow_fallback`] is true and a fallback is active or if
    /// the destination is in one of the [`super::System::dynamic_ranges`].
    pub alternative_font: Option<usize>,
    /// Position of the line number.
    pub place: LinePlace,
    /// Offset for line numbers with 1, 2, 3, 4 digits.
    pub offsets: [u32; 4],
    /// Spacing for line numbers with 1, 2, 3, 4 digits. First value should be
    /// 0.
    pub spacings: [u32; 4],
}

impl Default for Line {
    fn default() -> Self {
        Self {
            suppress: false,
            font: 0,
            alternative_font: None,
            place: Default::default(),
            offsets: [3, 2, 1, 1],
            spacings: [0, 2, 1, 1],
        }
    }
}

/// Destination text formatting settings.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Destination {
    /// Suppress the destination text on the display. Only the line number is displayed.
    pub suppress: bool,
    /// Destination fonts.
    pub fonts: DestinationFonts,
    /// Enables the conversion to upper lower case of destination texts.
    pub convert_upper_lower: UpperLowerConvert,
}

/// Display orientation (Front or Side).
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Orientation {
    #[default]
    Front,
    Side,
}

/// Position of the line number on the display.
///
/// More on the available display fields in [`Size`].
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum LinePlace {
    /// The line number is located in the line field and the destination text
    /// stays in the text field.
    Line,
    /// The line number is located in the text field together with the
    /// destination text.
    #[default]
    Text,
    /// The line number is located in the line field, but without a line number,
    /// the destination text is centered over line and text fields.
    Dynamic,
}

/// Destination fonts.
///
/// The fonts needs to exists in the font list in [`crate::group::Group`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DestinationFonts {
    /// Fonts for a single line text. Usually a set of 4 fonts.
    /// The order needs to be from largest to smallest font.
    pub one_line: Vec<usize>,
    /// Fonts for the first line of a two lines text.
    /// Usually a set of 2 fonts. The larger font needs to be the first.
    pub two_lines_up: Vec<usize>,
    /// Fonts for the second line of a two lines text.
    /// Usually a set of 2 fonts. The larger font needs to be the first.
    pub two_lines_lo: Vec<usize>,
}

impl Default for DestinationFonts {
    fn default() -> Self {
        Self {
            one_line: vec![0],
            two_lines_up: vec![0],
            two_lines_lo: vec![0],
        }
    }
}

/// Enables the conversion to upper lower case of destination texts
/// including word replacements from the change_word_list.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct UpperLowerConvert {
    /// If set to true, a single line destination text is converted.
    pub one_line: bool,
    /// If set to true, double line destination texts are converted.
    pub two_lines: bool,
}
