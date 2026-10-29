//! The preparation bucket boundary preserves retry classification and effects.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;

#[cfg(test)]
#[path = "cold_symbol_failure_matrix/exhaustive.rs"]
mod exhaustive;
