//! Selected-state merge reads do not promote an unconsulted generation cache.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;

#[cfg(test)]
#[path = "cold_merge_index_matrix/exhaustive.rs"]
mod exhaustive;
