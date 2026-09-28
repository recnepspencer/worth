mod authority;
mod categories;
mod kinds;
mod phase_one_compile_fail_targets;
mod phase_one_family_map;
mod phase_one_root_break_targets;

pub(crate) use authority::relational_source_truth_authority;
pub use categories::{
    RelationalSourceTruthAuthorityIdentity, RelationalSourceTruthBoundaryBridgedIdentity,
    RelationalSourceTruthDigestIdentityEvidence, RelationalSourceTruthExternalIdentityToken,
    RelationalSourceTruthProjectionIdentity,
};
pub use kinds::{
    RelationalBranchIdentityKind, RelationalCanonicalDigestIdentityBasis,
    RelationalCommitIdentityKind, RelationalEntityIdentityKind, RelationalRelationIdentityKind,
    RelationalSnapshotIdentityKind, RelationalVersionIdentityKind, RelationalWorkspaceIdentityKind,
};
pub use phase_one_compile_fail_targets::{
    relational_source_truth_identity_phase_one_compile_fail_targets,
    RelationalSourceTruthIdentityPhaseOneCompileFailTarget,
};
pub use phase_one_family_map::{
    relational_source_truth_identity_phase_one_families,
    RelationalSourceTruthIdentityPhaseOneFamily,
};
pub use phase_one_root_break_targets::{
    relational_source_truth_identity_phase_one_root_break_targets,
    RelationalSourceTruthIdentityPhaseOneRootBreakTarget,
};
