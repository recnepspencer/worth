pub(crate) mod access;
mod authority;
pub(crate) mod commit;
pub mod data;
pub(crate) mod envelope_demand;
pub mod retention;

pub use access::{BoundedCanonicalCommitPatchDenial, HistoryAccess};
pub(crate) use access::{CommitAncestryInspection, CommitAncestryPosture};
pub use access::{
    RelationalAuthoredEntityIndexDefinition, RelationalAuthoredManagedSourceImpact,
    RelationalAuthoredManagedSourceMaterial, RelationalAuthoredManagedSourcePayloads,
    RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteCursor,
    RelationalAuthoredWriteDenial, RelationalAuthoredWriteOccurrence, RelationalAuthoredWritePage,
    RelationalAuthoredWritePageLimits, RelationalAuthoredWriteScope,
    RelationalAuthoredWriteSelector, RelationalPreparedAuthoredWritePage,
};
pub use access::{
    RelationalCanonicalRecordHistoryCursor, RelationalPreparedRecordHistoryPage,
    RelationalRecordHistoryChange, RelationalRecordHistoryDenial, RelationalRecordHistoryPage,
    RelationalRecordHistoryPageLimits, RelationalRecordHistorySelection,
    RelationalRecordHistorySelectionLimits, RelationalRecordHistorySubject,
};
pub use authority::{HistoryAuthority, OutputOriginHeaderCorruptionForTest};
pub(crate) use commit::{
    RelationalCommitArtifact, RelationalCommitAuthoritativeAllocationKind, RelationalCommitCatalog,
};
pub use commit::{
    RelationalCommitArtifactDenial, RelationalCommitCatalogAppendDenial,
    RelationalCommitCatalogEntry, RelationalCommitIdentity, RelationalCommitParentage,
    RelationalCommitParentageDenial, RelationalCommitRootDescriptor,
    RelationalRetainedSelectionArtifactDenial,
};

pub(crate) use commit::RelationalCommitEnvelopeResidency;
#[cfg(test)]
pub(crate) use commit::{RelationalCommitIndexEntry, RetainedCanonicalIndex};
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
