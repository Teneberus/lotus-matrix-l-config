//! Configurable case conversion for destination display text.

use serde::{Deserialize, Serialize};

/// Settings shared by single-text and paired-text case conversion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ToUpperLowerConfig {
    /// Removes leading ASCII spaces (`U+0020`) before case conversion.
    pub trim_leading_spaces: bool,
    /// Removes trailing ASCII spaces (`U+0020`) before case conversion.
    pub trim_trailing_spaces: bool,
    /// Uppercases the first character after any leading ASCII spaces if it is an
    /// ASCII lowercase letter. Index `0` applies to a single text or the first
    /// text of a pair, and index `1` applies to the second text of a pair.
    pub start_with_uppercase: [bool; 2],
    /// Uppercases the first character of words that have no replacement match.
    pub convert_first_chars: bool,
    /// Marks the following uppercase letters for lowercasing, typically `_`.
    pub lowercase_marker: Option<char>,
    /// Marks the following lowercase letters for uppercasing, typically `?`.
    pub uppercase_marker: Option<char>,
    /// Characters that separate words during word case conversion.
    pub word_delimiters: Vec<char>,
    /// Characters that may immediately follow a replacement key. A key may also
    /// match at the end of the input.
    pub change_word_boundaries: Vec<char>,
}

impl Default for ToUpperLowerConfig {
    fn default() -> Self {
        Self {
            trim_leading_spaces: false,
            trim_trailing_spaces: true,
            start_with_uppercase: [true, false],
            convert_first_chars: false,
            lowercase_marker: None,
            uppercase_marker: None,
            word_delimiters: vec![' ', '/', '-', '+', '!', '.', '\r', '>', '<'],
            change_word_boundaries: vec![' ', '/', '-', '(', ',', '.', '\n'],
        }
    }
}

/// Extension trait for converting a single display text from any `AsRef<str>` type.
///
/// # Examples
///
/// ```
/// use lotus_matrix_l_config::{ToUpperLowerCase, ToUpperLowerConfig};
///
/// let config = ToUpperLowerConfig::default();
/// assert_eq!("bERLIN".to_upper_lower_case(&config, None, false, true), "Berlin");
/// ```
pub trait ToUpperLowerCase {
    /// Borrows the input and returns a converted `String` using [`ToUpperLowerConfig`].
    ///
    /// The conversion applies these steps in order.
    ///
    /// 1. Trim leading and trailing ASCII spaces (`U+0020`) according to
    ///    `trim_leading_spaces` and `trim_trailing_spaces`.
    /// 2. If `convert_upper_lower` is true, copy `word_delimiters` as-is and check
    ///    for a replacement at the next word start. The first nonempty key in
    ///    `change_word_list` that matches case-sensitively and is followed by the
    ///    end of the input or a `change_word_boundaries` character wins. Keys may
    ///    contain delimiters. Append the replacement as-is for this step and
    ///    resume scanning after the key. For words without a match, lowercase
    ///    everything after the first character and uppercase that first character
    ///    only if `convert_first_chars` is enabled. `None` disables replacements.
    /// 3. Apply `lowercase_marker` and then `uppercase_marker`, when configured.
    ///    Each pass removes its marker and converts the following run of characters
    ///    up to the first character whose case would stay the same. Word and marker
    ///    case conversion handles ASCII letters, `ÄÖÜäöü`, and their IBIS equivalents.
    /// 4. If `convert_chars` is true, decode `[`, `\`, `]`, `{`, `|`, and `}` as
    ///    `Ä`, `Ö`, `Ü`, `ä`, `ö`, and `ü`, respectively, and decode `^` and `~` as `ß`.
    /// 5. If `config.start_with_uppercase[0]` is true, uppercase the first character
    ///    after any leading ASCII spaces if it is an ASCII lowercase letter.
    fn to_upper_lower_case(
        &self,
        config: &ToUpperLowerConfig,
        change_word_list: Option<&Vec<(String, String)>>,
        convert_chars: bool,
        convert_upper_lower: bool,
    ) -> String;
}

impl<T: AsRef<str> + ?Sized> ToUpperLowerCase for T {
    fn to_upper_lower_case(
        &self,
        config: &ToUpperLowerConfig,
        change_word_list: Option<&Vec<(String, String)>>,
        convert_chars: bool,
        convert_upper_lower: bool,
    ) -> String {
        convert_line_at(
            self.as_ref(),
            0,
            config,
            change_word_list,
            convert_chars,
            convert_upper_lower,
        )
    }
}

/// Extension trait for converting two already separated display texts in `[T; 2]`
/// where `T` implements `AsRef<str>`.
///
/// # Examples
///
/// ```
/// use lotus_matrix_l_config::{ToUpperLowerCasePair, ToUpperLowerConfig};
///
/// let config = ToUpperLowerConfig::default();
/// assert_eq!(
///     ["bERLIN", "tEGEL"].to_upper_lower_case(&config, None, false, true),
///     ["Berlin", "tegel"],
/// );
/// ```
pub trait ToUpperLowerCasePair {
    /// Borrows the input array and returns the converted texts in the same order.
    ///
    /// Each text follows the steps in [`ToUpperLowerCase::to_upper_lower_case`].
    /// The first text uses `config.start_with_uppercase[0]` and the second uses
    /// `config.start_with_uppercase[1]`. Both use the same replacement list and
    /// the same values for `convert_chars` and `convert_upper_lower`.
    fn to_upper_lower_case(
        &self,
        config: &ToUpperLowerConfig,
        change_word_list: Option<&Vec<(String, String)>>,
        convert_chars: bool,
        convert_upper_lower: bool,
    ) -> [String; 2];
}

impl<T: AsRef<str>> ToUpperLowerCasePair for [T; 2] {
    fn to_upper_lower_case(
        &self,
        config: &ToUpperLowerConfig,
        change_word_list: Option<&Vec<(String, String)>>,
        convert_chars: bool,
        convert_upper_lower: bool,
    ) -> [String; 2] {
        let [first, second] = self;
        [
            first.to_upper_lower_case(config, change_word_list, convert_chars, convert_upper_lower),
            convert_line_at(
                second.as_ref(),
                1,
                config,
                change_word_list,
                convert_chars,
                convert_upper_lower,
            ),
        ]
    }
}

fn convert_line_at(
    line: &str,
    line_index: usize,
    config: &ToUpperLowerConfig,
    change_word_list: Option<&Vec<(String, String)>>,
    convert_chars: bool,
    convert_upper_lower: bool,
) -> String {
    let mut remaining = line;
    if config.trim_leading_spaces {
        remaining = remaining.trim_start_matches(' ');
    }
    if config.trim_trailing_spaces {
        remaining = remaining.trim_end_matches(' ');
    }

    let mut output = if convert_upper_lower {
        convert_words(remaining, config, change_word_list.map(Vec::as_slice))
    } else {
        remaining.to_owned()
    };

    if let Some(marker) = config.lowercase_marker {
        output = apply_case_marker(&output, marker, lowercase_word_char);
    }
    if let Some(marker) = config.uppercase_marker {
        output = apply_case_marker(&output, marker, uppercase_word_initial);
    }

    if convert_chars {
        output = output.chars().map(decode_ibis_char).collect();
    }

    if config.start_with_uppercase[line_index]
        && let Some(index) = output.bytes().position(|byte| byte != b' ')
        && output.as_bytes()[index].is_ascii_lowercase()
    {
        output[index..=index].make_ascii_uppercase();
    }

    output
}

fn convert_words(
    mut remaining: &str,
    config: &ToUpperLowerConfig,
    change_word_list: Option<&[(String, String)]>,
) -> String {
    let mut output = String::with_capacity(remaining.len());

    while let Some(first) = remaining.chars().next() {
        if config.word_delimiters.contains(&first) {
            output.push(first);
            remaining = &remaining[first.len_utf8()..];
            continue;
        }

        if let Some((key_len, replacement)) = change_word_list
            .and_then(|rules| find_replacement(remaining, rules, &config.change_word_boundaries))
        {
            output.push_str(replacement);
            remaining = &remaining[key_len..];
            continue;
        }

        output.push(if config.convert_first_chars {
            uppercase_word_initial(first)
        } else {
            first
        });

        remaining = &remaining[first.len_utf8()..];
        let word_end = remaining
            .find(|ch| config.word_delimiters.contains(&ch))
            .unwrap_or(remaining.len());
        output.extend(remaining[..word_end].chars().map(lowercase_word_char));
        remaining = &remaining[word_end..];
    }

    output
}

fn find_replacement<'rules>(
    text: &str,
    change_word_list: &'rules [(String, String)],
    boundaries: &[char],
) -> Option<(usize, &'rules str)> {
    change_word_list.iter().find_map(|(key, replacement)| {
        if key.is_empty() {
            return None;
        }

        let rest = text.strip_prefix(key.as_str())?;
        rest.chars()
            .next()
            .is_none_or(|next| boundaries.contains(&next))
            .then_some((key.len(), replacement.as_str()))
    })
}

fn lowercase_word_char(ch: char) -> char {
    match ch {
        'Ä' => 'ä',
        'Ö' => 'ö',
        'Ü' => 'ü',
        '[' => '{',
        '\\' => '|',
        ']' => '}',
        _ => ch.to_ascii_lowercase(),
    }
}

fn apply_case_marker(text: &str, marker: char, convert: fn(char) -> char) -> String {
    let mut output = String::with_capacity(text.len());
    let mut active = false;

    for ch in text.chars() {
        if ch == marker {
            active = true;
        } else if active {
            let converted = convert(ch);
            active = converted != ch;
            output.push(converted);
        } else {
            output.push(ch);
        }
    }

    output
}

fn uppercase_word_initial(ch: char) -> char {
    match ch {
        'ä' => 'Ä',
        'ö' => 'Ö',
        'ü' => 'Ü',
        '{' => '[',
        '|' => '\\',
        '}' => ']',
        _ => ch.to_ascii_uppercase(),
    }
}

fn decode_ibis_char(ch: char) -> char {
    match ch {
        '[' => 'Ä',
        '\\' => 'Ö',
        ']' => 'Ü',
        '{' => 'ä',
        '|' => 'ö',
        '}' => 'ü',
        '^' | '~' => 'ß',
        _ => ch,
    }
}
