mod access;
mod authority;
mod commit;
pub mod data;
pub mod retention;

pub(crate) use access::relevant_writes::{
    admit_work as admit_authored_write_work, ordered_visit as visit_authored_write_lookup,
    run as run_authored_write_stage, visit as visit_authored_write,
};
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
    RelationalCommitCatalogEnvelopeAppendDenial,
};
pub use commit::{
    RelationalCommitArtifactDenial, RelationalCommitCatalogAppendDenial,
    RelationalCommitCatalogEntry, RelationalCommitIdentity, RelationalCommitParentage,
    RelationalCommitParentageDenial, RelationalCommitRootDescriptor,
    RelationalRetainedSelectionArtifactDenial,
};

pub(crate) use commit::{RelationalCommitEnvelopeResidency, RelationalCommitIndexEntry};

pub(crate) use access::relevant_writes::posting_key::AuthoredWritePostingKey;

pub(crate) use access::relevant_writes::source_impact;
