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
//!
//! ## Quickstart
//!
//! [`Srcinfo`] is the main type for this crate.
//!
//! ```
//! # use srcinfo::Error;
//! use srcinfo::{Srcinfo, ArchVec};
//!
//! # fn test() -> Result<(), Error> {
//! // Create a srcinfo from a string
//! let srcinfo: Srcinfo = "
//! pkgbase = example
//! pkgver = 1.5.0
//! pkgrel = 5
//!
//! pkgname = example".parse()?;
//!
//! // Or a file
//! # let srcinfo = Srcinfo::parse_file("tests/srcinfo/libc++")?;
//! let srcinfo = Srcinfo::parse_file(".SRCINFO")?;
//!
//! // Reading global fields
//! // These fields were declared at the top of the PKGBUILD but may be overridden per package
//! println!("srcinfo {}-{}:", srcinfo.pkgbase(), srcinfo.version());
//!
//! // Print header comment
//! for comment in srcinfo.comment().lines() {
//!     println!("comment: {}", comment);
//! }
//!
//! println!("url: {}", srcinfo.url().unwrap_or("none"));
//! for arch in srcinfo.arch() {
//!     println!("arch: {}", arch);
//! }
//!
//! // reading makedepends and makedepends_$ARCH fields
//! for depends_arch in srcinfo.makedepends() {
//!     for depend in depends_arch.all() {
//!         match depends_arch.arch() {
//!             Some(arch) => println!("depend_{}: {}", arch, depend),
//!             None => println!("depend: {}", depend),
//!         }
//!     }
//! }
//!
//! // Iterate through all the packages in this srcinfo
//! for pkg in srcinfo.pkgs() {
//!     println!("pkg: {}", pkg.pkgname());
//! }
//!
//! // Get a specific package from the .SRCINFO
//! let pkg = srcinfo.pkg("libc++").unwrap();
//! println!("pkg: {}", pkg.pkgname());
//!
//! // Get the architectures of the package (may differ from the global architecture)
//! for arch in pkg.arch() {
//!     println!("{} arch: {}", pkg.pkgname(), arch);
//! }
//!
//! // Get the depends of an x86_64 system
//! // This includes the `depends` and `depends_x86_64` fields
//! for depend in ArchVec::active(pkg.depends(), "x86_64") {
//!     println!("depend: {}", depend);
//! }
//!
//! // Convert the .SRCINFO back into a string
//! // the new sring will semanticly match the original .SRCINFO
//! // but field order and whitespace will change, comments will be removed
//! let srcinfo = srcinfo.to_string();
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
mod error;
mod fmt;
mod parse;
mod srcinfo;

pub use crate::error::*;
pub use crate::srcinfo::*;
