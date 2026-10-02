mod basis;
mod contract_preparation;
mod digest_entry;
mod digest_slots;
mod equivalence;
mod export;
mod front_doors;
mod identity_preparation;
mod locator_preparation;
mod mask_preparation;
mod mismatch;
mod patch_preparation;
mod production_readiness;
mod readiness;
mod state_preparation;
mod value_lowering;

pub use value_lowering::canonical_basis_value_for_aspect_value;

pub use basis::{
    prepare_canonical_basis_bundle, prepare_canonical_basis_sequence,
    prepare_owned_canonical_basis_sequence_admitted, CanonicalBasisBundle,
    CanonicalBasisConstructionDenial, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryId, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisPreparationStop, CanonicalBasisReadinessProofs, CanonicalBasisReady,
    CanonicalBasisReadyArtifact, CanonicalBasisSequence, CanonicalBasisValue,
    CanonicalBundleReadinessProofs, CanonicalBundleReady, CanonicalBundleReadyArtifact,
    CanonicalComparisonReadinessProofs, CanonicalComparisonReady,
    CanonicalDigestDerivationReadinessProofs, CanonicalDigestDerivationReady,
    CanonicalDigestInputShapeBound, CanonicalDomainCoherence, CanonicalEquivalenceBasisDeclared,
    CanonicalExportManifestBound, CanonicalExportReadinessProofs, CanonicalExportReady,
    CanonicalFloatWidth, CanonicalIntegerWidth, CanonicalMismatchLociBound,
    CanonicalProductionReadinessCertified, CanonicalProductionTestReady, CanonicalRuleVersionBound,
    CanonicalizationCost, CanonicalizationCostObserved, CanonicalizationRuleVersion,
};
pub use contract_preparation::{
    aspect_contract_digest_preparation_basis, prepare_aspect_contract_for_canonical_basis,
    prepare_aspect_contract_for_digest,
};
pub use digest_entry::{
    CanonicalDigestAspectShapeKind, CanonicalDigestMaskMode, CanonicalDigestPreparationEntry,
};
pub use digest_slots::{
    admit_canonical_bundle_digest_derivation, admit_canonical_bundle_digest_derivation_with_budget,
    admit_canonical_export_digest_derivation, admit_canonical_export_digest_derivation_with_budget,
    admit_canonical_sequence_digest_derivation,
    admit_canonical_sequence_digest_derivation_with_budget, derive_canonical_digest,
    derive_canonical_digest_with_admission, CanonicalDerivedDigest, CanonicalDigestAdmissionStop,
    CanonicalDigestAlgorithmId, CanonicalDigestAlgorithmMetadata, CanonicalDigestAlgorithmSlot,
    CanonicalDigestBasisBundle, CanonicalDigestBasisSequence, CanonicalDigestDerivationDenial,
    CanonicalDigestDerivationInput, CanonicalDigestDerivationReadyArtifact,
    CanonicalDigestInputDomain, CanonicalDigestInputEvidence, CanonicalDigestInputId,
    CanonicalDigestInputShape, CanonicalDigestMetadata, CanonicalDigestOutputShape,
    CanonicalDigestValue, CanonicalDigestWorkBudget, CanonicalDigestWorkEvidence,
    CanonicalDomainBundleDigestAlgorithmSlot, CanonicalDomainBundleDigestInput,
    CanonicalExportBundleDigestAlgorithmSlot, CanonicalExportBundleDigestInput,
    CanonicalSingleSequenceDigestAlgorithmSlot, CanonicalSingleSequenceDigestInput,
};
pub use equivalence::{
    compare_canonical_basis, prepare_canonical_comparison, CanonicalComparisonInput,
    CanonicalComparisonOutcome, CanonicalComparisonReadyArtifact, CanonicalEquivalenceBasis,
    CanonicalEquivalentBasis,
};
pub use export::{
    bridge_canonical_export_trust_boundary, compare_canonical_exports,
    prepare_canonical_export_bundle, readmit_canonical_export_after_boundary,
    BoundaryBridgedCanonicalExportArtifact, CanonicalExportBasisBundle,
    CanonicalExportBasisSequence, CanonicalExportBundle, CanonicalExportComparisonOutcome,
    CanonicalExportDebt, CanonicalExportHarnessSeed, CanonicalExportManifest,
    CanonicalExportManifestMismatch, CanonicalExportManifestMismatchKind,
    CanonicalExportManifestRow, CanonicalExportReadmissionAuthority, CanonicalExportReadyArtifact,
    CanonicalProducerShape,
};
pub use front_doors::{
    canonicalization, CanonicalBasisFrontDoor, CanonicalBasisVersionStep,
    CanonicalComparisonBasisStep, CanonicalComparisonFrontDoor, CanonicalComparisonRightStep,
    CanonicalDigestFrontDoor, CanonicalExportBasisStep, CanonicalExportFrontDoor,
    CanonicalExportNameStep, CanonicalExportShapeStep, CanonicalReadinessFrontDoor,
    CanonicalizationFrontDoor,
};
pub use identity_preparation::{
    identity_canonical_basis_entries, prepare_identity_for_canonical_basis, CanonicalIdentityInput,
};
pub use locator_preparation::{
    diagnostic_mask_locator_canonical_basis_entries, locator_canonical_basis_entries,
    mutation_mask_locator_canonical_basis_entries, prepare_locator_for_canonical_basis,
    projection_mask_locator_canonical_basis_entries, CanonicalLocatorInput,
};
pub use mask_preparation::{
    aspect_mask_digest_preparation_basis, prepare_aspect_mask_for_canonical_basis,
    prepare_aspect_mask_for_digest, DigestPreparationMaskMode,
};
pub use mismatch::{CanonicalMismatchBasis, CanonicalMismatchKind};
pub use patch_preparation::{
    aspect_patch_digest_preparation_basis, prepare_aspect_patch_for_canonical_basis,
    prepare_aspect_patch_for_digest,
};
pub use production_readiness::{
    canonical_milestone2_production_readiness_report,
    certify_canonical_milestone2_production_readiness, require_canonical_production_test_readiness,
    CanonicalCertifiedSurface, CanonicalCertifiedSurfaceEvidence, CanonicalCompileFailBoundary,
    CanonicalCostCounterEvidence, CanonicalFixtureManifestEvidence,
    CanonicalGoldenArtifactEvidence, CanonicalHarnessExpansionPoint, CanonicalMilestone2PhaseGate,
    CanonicalPhaseGateEvidence, CanonicalProductionReadinessAuthority,
    CanonicalProductionReadinessReport, CanonicalProductionReadinessScope,
    CanonicalProductionTestReadyArtifact, CanonicalPropertySeed, CanonicalPropertySeedEvidence,
    CanonicalResidualDebt, CanonicalRuntimeAssumption, CanonicalRuntimeNonAssumption,
    CanonicalSyntheticRuntimePressure,
};
pub use readiness::{
    DigestPreparationReady, DigestPreparationReadyAspectContract,
    DigestPreparationReadyAspectContractArtifact, DigestPreparationReadyAspectMask,
    DigestPreparationReadyAspectMaskArtifact, DigestPreparationReadyAspectPatch,
    DigestPreparationReadyAspectPatchArtifact, DigestPreparationReadyAspectState,
    DigestPreparationReadyAspectStateArtifact, Milestone2DigestReadinessNote,
};
pub use state_preparation::{
    aspect_state_digest_preparation_basis, prepare_aspect_state_for_canonical_basis,
    prepare_aspect_state_for_digest,
};
pub use value_lowering::{
    prepare_aspect_value_identity_basis, prepare_struct_aspect_value_identity_basis,
    CanonicalAspectValueIdentityBasis,
};

pub fn canonical_basis_sequence_material(sequence: &CanonicalBasisSequence) -> String {
    digest_slots::basis_sequence_material(sequence)
}

use crate::facade::ResponsibilityArea;

pub fn responsibility() -> ResponsibilityArea {
    ResponsibilityArea::new(
        "canonical_ordering_and_equality",
        "stable ordering, equality, and digest-preparation basis vocabulary",
        "final digest algorithms or cryptographic receipt construction",
    )
}
