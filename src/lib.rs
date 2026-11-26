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
//!     let config = Group::from_toml_str(&s)
//!         .inspect_err(|e| log::error!("Failed to parse config: {}", e))
//!         .unwrap_or_default();
//! };
//! ```

pub mod content;
pub mod group;

mod to_upper_lower;

pub use content::Content;
pub use group::Group;
pub use to_upper_lower::{ToUpperLowerCase, ToUpperLowerCasePair, ToUpperLowerConfig};
