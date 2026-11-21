//! Immutable physical addresses. A handle proves storage identity, never currentness.
use super::super::fact_key::FactPostingKey;
use super::super::mark_state::FullVerificationReason;

use im::OrdMap;
use im::OrdSet;
use std::sync::Arc;

#[derive(Debug)]
pub struct PostingProjection {
    pub id: u64,
    pub ordinals:
        OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
    pub source_len:
        usize,
    pub output_len:
        usize,
    pub requirement:
        Option<FullVerificationReason>,
}
impl PostingProjection {
    pub fn ordinals(
        &self,
    ) -> &OrdMap<Arc<FactPostingKey>, OrdSet<usize>> {
        &self.ordinals
    }
}

/// Allocation custody selected from the same exact settlement row.
#[derive(Clone, Debug)]
pub struct PostingComputationStorage
{
    pub projection:
        Arc<PostingProjection>,
}

mod court_doors_for_test;
