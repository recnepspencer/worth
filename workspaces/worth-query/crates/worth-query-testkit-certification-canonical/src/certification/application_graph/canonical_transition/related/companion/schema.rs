//! Stable companion meaning, unchanged across the codec cutover.
#[path = "schema/records.rs"]
mod storage_model;
pub use super::super::schema::CanonicalSchema;
pub use storage_model::*;
pub use worth_query_host::facade::{declaration::application_schema::*, *};
