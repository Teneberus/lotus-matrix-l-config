//! This crate provides the special pis configuration structure for all
//! destinations displays that want to use it.
//! It also provides some useful functions that make use of the available
//! parameters, for example an highly configurable upper lower conversion
//! algorithm.
//!
//! Hopefully it is possible to simulate the destination displays in
//! LOTUS-Simulator as close as possible to the reality with the provides
//! configuration possibilities.
//!
//! Read from group strings:
//!
//! ```ignore
//! use lotus_matrix_l_config::Group;
//! use lotus_script::{pis::PisSpGroup, prelude::*};
//!
//! if let Some(content_id) = PisSpGroup::get_content_id("MATRIX_L") {
//!     let s = PisSpGroup::get_group_strings(content_id).replace("\t", "\n");
//!
//!     let config = Group::from_str(&s)
//!         .inspect_err(|e| log::error!("Failed to parse config: {}", e))
//!         .unwrap_or_default();
//! };
//! ```
//!
//! Both configuration types accept TOML and JSON.
//!
//! ```
//! use lotus_matrix_l_config::{Content, Group};
//!
//! let group = Group::default();
//! let json = group.to_json()?;
//! assert_eq!(Group::from_str::<Group>(&json)?, group);
//!
//! let content = Content::default();
//! let toml = content.to_toml()?;
//! assert_eq!(Content::from_slice::<Content>(toml.as_bytes())?, content);
//!
//! let error = Content::from_str::<Content>("{").unwrap_err().to_string();
//! assert!(error.contains("TOML:") && error.contains("JSON:"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod content;
pub mod group;

mod to_upper_lower;

pub use content::Content;
pub use group::Group;
pub use to_upper_lower::{ToUpperLowerCase, ToUpperLowerCasePair, ToUpperLowerConfig};

/// Deserialization failed in both supported formats.
#[derive(Debug)]
pub struct DeserializeError {
    /// The error returned by the TOML parser.
    pub toml: toml::de::Error,
    /// The error returned by the JSON parser.
    pub json: serde_json::Error,
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TOML: {}\nJSON: {}", self.toml, self.json)
    }
}

impl std::error::Error for DeserializeError {}

fn deserialize_config<'de, T: serde::Deserialize<'de>>(
    input: &'de [u8],
) -> Result<T, DeserializeError> {
    toml::from_slice(input).or_else(|toml| {
        serde_json::from_slice(input).map_err(|json| DeserializeError { toml, json })
    })
}
