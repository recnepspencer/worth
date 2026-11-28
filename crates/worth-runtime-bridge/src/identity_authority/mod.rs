mod admission;
mod authority;
mod categories;
mod kinds;
mod phase_one_compile_fail_targets;
mod phase_one_family_map;
mod phase_one_root_break_targets;

pub(crate) use admission::{
    admit_bridge_truth_authority_identity, admit_bridge_truth_authority_identity_for_kind,
};
pub use admission::{
    bridge_truth_digest_identity_evidence_from_external_token,
    bridge_truth_external_identity_token, bridge_truth_projection_identity_from_external_token,
};
pub(crate) use authority::bridge_truth_authority;
pub use authority::BridgeTruthAuthority;
pub use categories::{
    BridgeTruthAuthorityIdentity, BridgeTruthBoundaryBridgedIdentity,
    BridgeTruthDigestIdentityEvidence, BridgeTruthExternalIdentityToken,
    BridgeTruthProjectionIdentity,
};
pub use kinds::{
    BridgeBranchIdentityKind, BridgeCanonicalDigestIdentityBasis, BridgeCausalEnvelopeIdentityKind,
    BridgeCausalReferenceIdentityKind, BridgeCommitIdentityKind,
    BridgeEvidenceReferenceIdentityKind, BridgePatchIdentityKind, BridgePreviewSessionIdentityKind,
    BridgeReceiptIdentityKind, BridgeRecordIdentityKind, BridgeRetainedMappingDigestIdentityBasis,
    BridgeRetainedMappingIdentityKind, BridgeSnapshotIdentityKind,
    BridgeWritebackDeclarationIdentityKind,
};
pub use phase_one_compile_fail_targets::{
    bridge_truth_identity_phase_one_compile_fail_targets,
    BridgeTruthIdentityPhaseOneCompileFailTarget,
};
pub use phase_one_family_map::{
    bridge_truth_identity_phase_one_families, BridgeTruthIdentityPhaseOneFamily,
};
pub use phase_one_root_break_targets::{
    bridge_truth_identity_phase_one_root_break_targets, BridgeTruthIdentityPhaseOneRootBreakTarget,
};
