use worth_relational_contracts::internal_port as contracts;
pub mod digest;
pub mod lineage_authority;
pub mod parity;
pub mod verification;

pub use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
pub use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::replay_record::RelationalReplayRecord;
pub use contracts::history::data::replay_version::ReplaySchemaVersion;
pub(crate) use digest::digest_derived_index_summary;
pub(crate) use digest::digest_derived_index_surface;
pub use digest::digest_branch_head_summary;
pub use digest::digest_branch_head_surface;
pub use digest::digest_canonical_patch_summary;
pub use digest::digest_canonical_patch_surface;
pub use digest::digest_diagnostics_summary;
pub use digest::digest_diagnostics_surface;
pub use digest::digest_history_summary;
pub use digest::digest_history_surface;
pub use digest::digest_lineage_decision_log_surface;
pub use digest::digest_lineage_decision_summary;
pub use digest::digest_lineage_event_batch_surface;
pub use digest::digest_lineage_event_summary;
pub use digest::digest_schema_continuation_descriptor;
pub use digest::digest_schema_continuation_summary;
pub use digest::digest_schema_lineage_summary;
pub use digest::digest_schema_reconciliation_descriptor;
pub use digest::digest_schema_reconciliation_summary;
pub use digest::digest_schema_transition_descriptor;
pub use digest::digest_schema_transition_summary;
pub use digest::digest_snapshot_summary;
pub use digest::digest_snapshot_surface;
pub use digest::{
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
pub use contracts::replay::data::replay_errors::{ReplayError, ReplayFailureClass};
pub use verification::{
    RelationalReplayOutcome, RelationalReplayRequest, ReplayExecutionMode, ReplayMismatch,
    ReplayMismatchClass, ReplayObservableSurface, ReplaySnapshotSurface, ReplayVerificationLayer,
    ReplayVerificationMode, ReplayVerificationPlan,
};
