use super::CanonicalCommitEnvelope;
use crate::transactions::data::MutationIntent;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CanonicalCommitEnvelopeAllocationInventory {
    pub(crate) authoritative_nested_bytes: u64,
    pub(crate) diagnostic_bytes: u64,
    pub(crate) optional_cache_bytes: u64,
}

impl CanonicalCommitEnvelope {
    pub(crate) fn allocation_inventory(&self) -> CanonicalCommitEnvelopeAllocationInventory {
        let commit_parents = vector_capacity_bytes(&self.commit.parents);
        let merge_parent_storage = vector_capacity_bytes(&self.merge_parent_branches);
        let merge_parent_names = self
            .merge_parent_branches
            .iter()
            .map(|branch| branch.0.capacity() as u64)
            .sum::<u64>();
        let merged_intents = vector_capacity_bytes(&self.merged_plan.merged_intents)
            .saturating_add(
                self.merged_plan
                    .merged_intents
                    .iter()
                    .map(MutationIntent::owned_allocation_capacity_bytes)
                    .sum(),
            );
        let patch_storage = vector_capacity_bytes(&self.patch.authoritative_record_patches);
        let patch_nested_storage = self
            .patch
            .authoritative_record_patches
            .iter()
            .map(|patch| {
                vector_capacity_bytes(&patch.semantic_changes)
                    .saturating_add(
                        patch
                            .semantic_changes
                            .iter()
                            .map(|change| change.owned_allocation_capacity_bytes())
                            .sum(),
                    )
                    .saturating_add(patch.authoritative_patch.owned_allocation_capacity_bytes())
                    .saturating_add(patch.detail.owned_allocation_capacity_bytes())
            })
            .sum::<u64>();
        let authoritative_nested_bytes = (self.commit.branch_id.0.capacity() as u64)
            .saturating_add(optional_canonical_nested_bytes(
                &self.required_output_origin,
            ))
            .saturating_add(optional_canonical_nested_bytes(
                &self.required_output_materialization,
            ))
            .saturating_add(commit_parents)
            .saturating_add(self.branch_context.0.capacity() as u64)
            .saturating_add(optional_canonical_nested_bytes(
                &self.branch_cell_checkpoint,
            ))
            .saturating_add(merge_parent_storage)
            .saturating_add(merge_parent_names)
            .saturating_add(vector_capacity_bytes(&self.merge_base_commits))
            .saturating_add(self.schema_authority.owned_allocation_capacity_bytes())
            .saturating_add(merged_intents)
            .saturating_add(vector_capacity_bytes(&self.record_allocations))
            .saturating_add(patch_storage)
            .saturating_add(patch_nested_storage)
            .saturating_add(self.published_lineage().owned_allocation_capacity_bytes())
            // These optional authority records own deeply nested vectors and
            // strings across several domains. Their canonical encoding is the
            // owner-defined variable-size footprint; the fixed Option/object
            // storage is already charged by the envelope object allocation.
            .saturating_add(optional_canonical_nested_bytes(&self.strategy_artifacts))
            .saturating_add(optional_canonical_nested_bytes(
                &self.merge_execution_authority,
            ))
            .saturating_add(optional_canonical_nested_bytes(&self.schema_transition))
            .saturating_add(optional_canonical_nested_bytes(
                &self.schema_continuation_descriptor,
            ))
            .saturating_add(optional_canonical_nested_bytes(
                &self.schema_reconciliation_descriptor,
            ));
        CanonicalCommitEnvelopeAllocationInventory {
            authoritative_nested_bytes,
            diagnostic_bytes: self.diagnostics_summary.owned_allocation_capacity_bytes(),
            optional_cache_bytes: self
                .derived_index_artifacts
                .owned_allocation_capacity_bytes(),
        }
    }
}

fn optional_canonical_nested_bytes<T: serde::Serialize>(value: &Option<T>) -> u64 {
    value
        .as_ref()
        .map(|value| {
            rmp_serde::to_vec(value)
                .expect("canonical authority fields are serializable")
                .len() as u64
        })
        .unwrap_or(0)
}

fn vector_capacity_bytes<T>(values: &Vec<T>) -> u64 {
    (values.capacity() as u64).saturating_mul(std::mem::size_of::<T>() as u64)
}
