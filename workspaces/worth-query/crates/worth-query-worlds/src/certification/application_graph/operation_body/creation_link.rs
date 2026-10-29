//! A body creates rows and their graph linkage atomically through native owners.
pub use super::*;
#[path = "creation_link/portable.rs"]
pub mod portable;
#[path = "creation_link/query.rs"]
pub mod query;
