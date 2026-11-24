//! This crate provides the special pis configuration structure for all
//! destinations displays that want to use it.
//! It also provides some useful functions that make use of the available
//! parameters, for example an highly configurable upper lower conversion
//! algorithm.
//!
//! Hopefully it is possible to simulate the destination displays in
//! LOTUS-Simulator as close as possible to the reality with the provides
//! configuration possibilities.

pub mod content;
pub mod group;

mod to_upper_lower;

pub use content::Content;
pub use group::Group;
pub use to_upper_lower::{ToUpperLowerCase, ToUpperLowerCasePair, ToUpperLowerConfig};
