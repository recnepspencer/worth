mod correlation;
mod denial;
mod foundational_roles;
mod physical_read_stability_authority;
mod semantic_visibility_reference;

pub use correlation::{
    correlate_semantic_visibility_with_physical_snapshot, PhysicalSnapshotCorrelation,
    SemanticCorrelationCapability,
};
pub use denial::{
    deny_semantic_visibility_as_physical_stability, PhysicalSemanticBoundaryDenial,
    PhysicalSemanticBoundaryOutcome, SemanticVisibilityCannotMintPhysicalStability,
};
pub use foundational_roles::PhysicalSemanticBoundaryRoleEvidence;
#[cfg(any(test, feature = "certification-authority"))]
pub use physical_read_stability_authority::physical_read_stability_authority_for_certification_test;
pub use physical_read_stability_authority::{
    admit_post_compaction_read_stability_authority,
    admit_post_publication_read_stability_authority, PhysicalReadStabilityAuthority,
    PhysicalReadStabilityCorrelationBasis,
};
pub use semantic_visibility_reference::{
    SemanticVisibilityReference, SemanticVisibilityReferenceKind,
};
