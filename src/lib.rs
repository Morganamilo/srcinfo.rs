//! # Srcinfo
//!
//! Srcinfo is a parser for makepkg's .SRCINFO file format.
//!
//! Srcinfo focuses on correctness of parsing, especially
//! with split packages and architecture specific fields.
//!
//! Srcinfo only aims to parse. This crate does not attempt to
//! perform any version comparison, dependency checking or any other
//! extra functionality.

#![warn(missing_docs)]
mod error;
mod parse;
mod srcinfo;

pub use crate::error::*;
pub use crate::srcinfo::*;
