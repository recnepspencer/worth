mod artifact;
mod catalog;
mod identity;
pub(crate) mod original_payload;
mod parentage;
pub(crate) mod payload_commitment;

pub(crate) use artifact::RelationalCommitAuthoritativeAllocationKind;
pub use artifact::RelationalRetainedSelectionArtifactDenial;
pub(crate) use artifact::{
    CapturedOriginalCanonicalInputs, OriginalCanonicalInputs, RelationalCommitArtifact,
};
pub use artifact::{RelationalCommitArtifactDenial, RelationalCommitRootDescriptor};
pub(crate) use catalog::{RelationalCommitCatalog, RelationalCommitCatalogEnvelopeAppendDenial};
pub use catalog::{RelationalCommitCatalogAppendDenial, RelationalCommitCatalogEntry};
pub use identity::RelationalCommitIdentity;
pub use parentage::{RelationalCommitParentage, RelationalCommitParentageDenial};

pub(crate) mod envelope_residency;
#[cfg(test)]
mod envelope_residency_tests;
mod index_entry;
pub(crate) use envelope_residency::{RelationalCommitEnvelopeResidency, RetiredCanonicalPlacement};
pub(crate) use index_entry::RelationalCommitIndexEntry;

mod retained_index;
pub(crate) use retained_index::RetainedCanonicalIndex;

pub(crate) mod artifact_denial;
