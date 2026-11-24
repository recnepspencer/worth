//! Source-backed payload for one retained entity selection.
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::retained_entity_selection::CanonicalRetainedEntitySelection;
use worth_relational_storage::internal_port::storage::substrate::record_arena::values::extra::EntityExtra;
use worth_relational_storage::internal_port::storage::substrate::storage_allocation::StorageAllocation;
use std::sync::Arc;

/// Prepared only from the actual retained record, never from canonical data.
#[derive(Debug, Clone)]
pub struct PreparedRetainedEntityPayload {
    // Kept for its owner-governed retention, including pending publication.
    pub _source: AdmittedRelationalBranchBasis,
    pub meaning: Arc<CanonicalRetainedEntitySelection>,
    pub payload: StorageAllocation<EntityExtra>,
}
