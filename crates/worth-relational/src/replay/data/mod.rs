mod digest;
mod lineage_authority;
mod parity;
pub(crate) use worth_relational_contracts::internal_port::replay::data::replay_errors;
mod verification;

pub use crate::history::data::{
    CanonicalCommitEnvelope, RelationalReplayRecord, ReplaySchemaVersion,
};
pub(crate) use digest::{
    digest_branch_head_summary, digest_branch_head_surface, digest_canonical_patch_summary,
    digest_canonical_patch_surface, digest_derived_index_summary, digest_derived_index_surface,
    digest_diagnostics_summary, digest_diagnostics_surface, digest_history_summary,
    digest_history_surface, digest_lineage_decision_log_surface, digest_lineage_decision_summary,
    digest_lineage_event_batch_surface, digest_lineage_event_summary,
    digest_schema_continuation_descriptor, digest_schema_continuation_summary,
    digest_schema_lineage_summary, digest_schema_reconciliation_descriptor,
    digest_schema_reconciliation_summary, digest_schema_transition_descriptor,
    digest_schema_transition_summary, digest_snapshot_summary, digest_snapshot_surface,
    digest_strategy_replay_descriptor, digest_strategy_replay_summary,
};
#[cfg(test)]
pub(crate) use digest::{
    digest_diagnostics_batch_surface, digest_patch_batch_surface,
    digest_schema_transition_decision, digest_subscriber_boundary_cdc_surface,
    digest_subscriber_continuation_counter_pair, digest_subscriber_continuation_summary,
};
pub use lineage_authority::{
    CertifiedLineageSurfaceComparisonBasis, CertifiedLineageSurfaceDigest,
    LineageCertifiedSurfaceKind, ReplayAuthorityBasisKind, ReplayLineageAuthorityBasis,
    ReplayLineageDigestMode,
};
pub use parity::{
    DescriptorAuthorityKind, DescriptorComparisonBasis, DescriptorParityCheck,
    ReplaySurfaceAuthorityKind, ReplaySurfaceComparisonBasis, ReplaySurfaceParityCheck,
    VerifiedDescriptorDigest, VerifiedReplaySurfaceDigest,
};
pub use replay_errors::{ReplayError, ReplayFailureClass};
pub use verification::{
    RelationalReplayOutcome, RelationalReplayRequest, ReplayExecutionMode, ReplayMismatch,
    ReplayMismatchClass, ReplayObservableSurface, ReplaySnapshotSurface, ReplayVerificationLayer,
    ReplayVerificationMode, ReplayVerificationPlan,
};
