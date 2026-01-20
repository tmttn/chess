//! Chess opening book database and lookup.
//!
//! This crate provides functionality for storing, loading, and querying
//! chess opening databases. It includes built-in opening data and supports
//! custom opening books.

pub mod builtin;
pub mod database;
pub mod opening;

pub use database::OpeningDatabase;
pub use opening::{Opening, OpeningMove};
