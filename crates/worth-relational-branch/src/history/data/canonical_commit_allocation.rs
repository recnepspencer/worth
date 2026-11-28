use crate::internal_port::indexes::data::DerivedIndexArtifactsOwnerPort as _;
use worth_relational_contracts::internal_port::diagnostics::data::artifacts::RelationalDiagnosticArtifactOwnerPort as _;
use worth_relational_contracts::internal_port::publication::patch::data::patch_detail::PatchDetailOwnerPort as _;
use worth_relational_contracts::internal_port::publication::patch::data::published_authoritative_patch::PublishedAuthoritativePatchOwnerPort as _;
use worth_relational_contracts::internal_port::publication::patch::data::semantic_change::PublishedAuthoritativeAspectChangeOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::intents::mutation_intent::MutationIntentOwnerPort as _;
use super::{CanonicalCommitEnvelope, CanonicalCommitEnvelopeBody};
use crate::transactions::data::intents::mutation_intent::MutationIntent;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CanonicalCommitEnvelopeAllocationInventory {
    pub authoritative_nested_bytes: u64,
    pub diagnostic_bytes: u64,
    pub optional_cache_bytes: u64,
}

impl CanonicalCommitEnvelope {
    pub(crate) fn allocation_inventory(&self) -> CanonicalCommitEnvelopeAllocationInventory {
        let mut inventory = CanonicalCommitEnvelopeBody::allocation_inventory(self);
        // The same immutable image is shared by the live result and this
        // canonical artifact. Its retained body uses the existing canonical
        // nested-field footprint policy exactly once in this owner inventory.
        inventory.authoritative_nested_bytes = inventory
            .authoritative_nested_bytes
            .saturating_add(std::mem::size_of_val(self.semantic_result()) as u64)
            .saturating_add(canonical_nested_bytes(self.semantic_result()));
        inventory
    }
}

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn allocation_inventory(&self) -> CanonicalCommitEnvelopeAllocationInventory {
        let commit_parents = vector_capacity_bytes(&self.commit.parents);
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
            .saturating_add(
                self.caller_association
                    .as_ref()
                    .map_or(0, |value| value.as_bytes().len() as u64),
            )
            .saturating_add(optional_canonical_nested_bytes(
                &self.required_output_origin,
            ))
            .saturating_add(optional_canonical_nested_bytes(
                &self.producer_cascade_retirement,
            ))
            .saturating_add(optional_canonical_nested_bytes(
                &self.authored_cascade_retirement,
            ))
            .saturating_add(optional_canonical_nested_bytes(
                &self.authored_cascade_restoration,
            ))
            .saturating_add(optional_canonical_nested_bytes(
                &self.required_output_materialization,
            ))
            .saturating_add(self.authored_index_writes.unpaid_owned_bytes())
            .saturating_add(self.index_definition_delta.unpaid_owned_bytes())
            .saturating_add(commit_parents)
            .saturating_add(self.branch_context.0.capacity() as u64)
            .saturating_add(optional_canonical_nested_bytes(
                &self.branch_cell_checkpoint,
            ))
            // Schema backing is shared with the root authority, which inventories it once.
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
        .map(|value| canonical_nested_bytes(value))
        .unwrap_or(0)
}

fn canonical_nested_bytes<T: serde::Serialize>(value: &T) -> u64 {
    rmp_serde::to_vec(value)
        .expect(/* Proof: Artifact construction first successfully encodes this same immutable canonical envelope, including its mandatory semantic image. Cold demand verifies that encoding before exposing the Arc. The closed fields cannot introduce a new serialization refusal in this nested footprint projection. */ "canonical authority fields are serializable")
        .len() as u64
}

fn vector_capacity_bytes<T>(values: &Vec<T>) -> u64 {
    (values.capacity() as u64).saturating_mul(std::mem::size_of::<T>() as u64)
}

#[doc(hidden)]
pub trait CanonicalCommitEnvelopeBodyOwnerPort: Sized {
    fn allocation_inventory(&self) -> CanonicalCommitEnvelopeAllocationInventory;
}

impl CanonicalCommitEnvelopeBodyOwnerPort for CanonicalCommitEnvelopeBody {
    fn allocation_inventory(&self) -> CanonicalCommitEnvelopeAllocationInventory {
        self.allocation_inventory()
    }
}
