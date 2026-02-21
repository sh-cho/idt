//! IDT (ID Tool) - A fast, ergonomic CLI tool for working with various ID formats.
//!
//! This library provides functionality for generating, parsing, inspecting,
//! converting, and validating various identifier formats including:
//!
//! - UUID (all versions: v1, v3, v4, v5, v6, v7)
//! - ULID (Universally Unique Lexicographically Sortable Identifier)
//! - NanoID (compact URL-friendly unique ID)
//! - Snowflake (Twitter-style distributed ID)
//!
//! # Example
//!
//! ```rust
//! use idt::ids::{UuidGenerator, UlidGenerator};
//! use idt::core::id::IdGenerator;
//!
//! // Generate a UUIDv4
//! let uuid = UuidGenerator::v4().generate().unwrap();
//! println!("UUID: {}", uuid);
//!
//! // Generate a ULID
//! let ulid = UlidGenerator::new().generate().unwrap();
//! println!("ULID: {}", ulid);
//! ```

pub mod cli;
pub mod core;
pub mod ids;
pub mod utils;

// Re-export commonly used types
pub use core::EncodingFormat;
pub use core::error::{IdtError, Result};
pub use core::id::{IdGenerator, IdKind, InspectionResult, ParsedId, Timestamp, ValidationResult};
