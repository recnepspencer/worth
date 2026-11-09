//! Edges retain real endpoint relations; vertex-region changes own their bounds.
#[path = "schema/records.rs"]
mod storage_model;
use super::super::{
    bounds::{Coordinates, CoordinatesBinding},
    schema::{Catalog, Directory},
};
pub use storage_model::*;
