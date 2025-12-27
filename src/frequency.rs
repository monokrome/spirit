//! Frequency data structures and category registry.
//!
//! This module re-exports the generated frequency types and data.
//! All category data is generated at compile time from frequencies.toml.

// Include the generated frequency module
include!(concat!(env!("OUT_DIR"), "/frequency.rs"));
