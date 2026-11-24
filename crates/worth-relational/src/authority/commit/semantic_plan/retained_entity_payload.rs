//! Source-backed payload for one retained entity selection.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::CanonicalRetainedEntitySelection;
use crate::storage::substrate::{EntityExtra, StorageAllocation};
use std::sync::Arc;

/// Prepared only from the actual retained record, never from canonical data.
#[derive(Debug, Clone)]
pub(crate) struct PreparedRetainedEntityPayload {
    // Kept for its owner-governed retention, including pending publication.
    pub(crate) _source: AdmittedRelationalBranchBasis,
    pub(crate) meaning: Arc<CanonicalRetainedEntitySelection>,
    pub(crate) payload: StorageAllocation<EntityExtra>,
}
