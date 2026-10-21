//! Immutable physical addresses. A handle proves storage identity, never currentness.
use super::super::fact_key::FactPostingKey;
use super::super::mark_state::FullVerificationReason;

use im::OrdMap;
use im::OrdSet;
use std::sync::Arc;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) struct PostingProjection
{
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) id: u64,
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) ordinals:
        OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) source_len:
        usize,
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) output_len:
        usize,
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) requirement:
        Option<FullVerificationReason>,
}
impl PostingProjection {
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn ordinals(
        &self,
    ) -> &OrdMap<Arc<FactPostingKey>, OrdSet<usize>> {
        &self.ordinals
    }
}

/// Allocation custody selected from the same exact settlement row.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) struct PostingComputationStorage
{
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) projection:
        Arc<PostingProjection>,
}
