mod artifact;
mod catalog;
mod identity;
mod parentage;

pub(crate) use artifact::RelationalCommitAuthoritativeAllocationKind;
pub(crate) use artifact::{OriginalCanonicalInputs, RelationalCommitArtifact};
pub use artifact::{RelationalCommitArtifactDenial, RelationalCommitRootDescriptor};
pub(crate) use catalog::{RelationalCommitCatalog, RelationalCommitCatalogEnvelopeAppendDenial};
pub use catalog::{RelationalCommitCatalogAppendDenial, RelationalCommitCatalogEntry};
pub use identity::RelationalCommitIdentity;
pub use parentage::{RelationalCommitParentage, RelationalCommitParentageDenial};

mod envelope_residency;
#[cfg(test)]
mod envelope_residency_tests;
mod index_entry;
pub(crate) use envelope_residency::{RelationalCommitEnvelopeResidency, RetiredCanonicalPlacement};
pub(crate) use index_entry::RelationalCommitIndexEntry;

mod retained_index;
pub(crate) use retained_index::RetainedCanonicalIndex;
