use super::vocabulary::{
    FoundationalBoundaryEvidenceCertifiedSurface,
    FoundationalBoundaryEvidenceCertifiedSurfaceEvidence,
    FoundationalBoundaryEvidenceCompileFailBoundary, FoundationalBoundaryEvidenceGoldenArtifact,
    FoundationalBoundaryEvidenceHarnessExpansionPoint,
    FoundationalBoundaryEvidenceMilestone7PhaseGate, FoundationalBoundaryEvidencePhaseGateEvidence,
    FoundationalBoundaryEvidencePropertySeed, FoundationalBoundaryEvidencePropertySeedEvidence,
    FoundationalBoundaryEvidenceResidualDebt, FoundationalBoundaryEvidenceRuntimeAssumption,
    FoundationalBoundaryEvidenceRuntimeNonAssumption,
    FoundationalBoundaryEvidenceSyntheticRuntimePressure,
};
use crate::boundary_evidence_api::{
    boundary_evidence_public_surface_inventory, BoundaryEvidencePublicSurfaceEntry,
};

pub(super) fn certified_surfaces() -> Vec<FoundationalBoundaryEvidenceCertifiedSurface> {
    vec![
        FoundationalBoundaryEvidenceCertifiedSurface::PrimitiveCategoryAndRoleLaw,
        FoundationalBoundaryEvidenceCertifiedSurface::ProvenanceLayeringAndFreshnessLaw,
        FoundationalBoundaryEvidenceCertifiedSurface::ReceiptFamilyAndCloseoutTruth,
        FoundationalBoundaryEvidenceCertifiedSurface::LineageContinuityAndDivergence,
        FoundationalBoundaryEvidenceCertifiedSurface::SupportTruthRecoveryAndDebt,
        FoundationalBoundaryEvidenceCertifiedSurface::AttachmentMaterializationAndReadmission,
    ]
}

pub(super) fn synthetic_pressures() -> Vec<FoundationalBoundaryEvidenceSyntheticRuntimePressure> {
    vec![
        FoundationalBoundaryEvidenceSyntheticRuntimePressure::PrimitiveAdjacencyHostility,
        FoundationalBoundaryEvidenceSyntheticRuntimePressure::FreshnessDisclosureHostility,
        FoundationalBoundaryEvidenceSyntheticRuntimePressure::PlannedVersusExecutedSeparation,
        FoundationalBoundaryEvidenceSyntheticRuntimePressure::ReplayVersusHistoryMasqueradeRejection,
        FoundationalBoundaryEvidenceSyntheticRuntimePressure::SupportGradeOverclaimRejection,
        FoundationalBoundaryEvidenceSyntheticRuntimePressure::AttachmentScopeAndOrderingHostility,
        FoundationalBoundaryEvidenceSyntheticRuntimePressure::TrustBoundaryReadmissionWORTHry,
    ]
}

pub(super) fn certified_surface_evidence(
) -> Vec<FoundationalBoundaryEvidenceCertifiedSurfaceEvidence> {
    vec![
        FoundationalBoundaryEvidenceCertifiedSurfaceEvidence::new(
            FoundationalBoundaryEvidenceCertifiedSurface::PrimitiveCategoryAndRoleLaw,
            FoundationalBoundaryEvidenceSyntheticRuntimePressure::PrimitiveAdjacencyHostility,
            FoundationalBoundaryEvidenceCompileFailBoundary::PrimitiveNonSubstitution,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/primitives/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_evidence/primitives/locality_cannot_satisfy_category_api.rs"),
        ),
        FoundationalBoundaryEvidenceCertifiedSurfaceEvidence::new(
            FoundationalBoundaryEvidenceCertifiedSurface::ProvenanceLayeringAndFreshnessLaw,
            FoundationalBoundaryEvidenceSyntheticRuntimePressure::FreshnessDisclosureHostility,
            FoundationalBoundaryEvidenceCompileFailBoundary::ProvenanceFreshnessAndArtifactBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/provenance/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_evidence/provenance/provenance_builder_requires_freshness.rs"),
        ),
        FoundationalBoundaryEvidenceCertifiedSurfaceEvidence::new(
            FoundationalBoundaryEvidenceCertifiedSurface::ReceiptFamilyAndCloseoutTruth,
            FoundationalBoundaryEvidenceSyntheticRuntimePressure::PlannedVersusExecutedSeparation,
            FoundationalBoundaryEvidenceCompileFailBoundary::ReceiptPlanningVersusCompletedBoundarySeparation,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/receipts/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_evidence/receipts/planning_receipt_cannot_satisfy_completed_receipt_api.rs"),
        ),
        FoundationalBoundaryEvidenceCertifiedSurfaceEvidence::new(
            FoundationalBoundaryEvidenceCertifiedSurface::LineageContinuityAndDivergence,
            FoundationalBoundaryEvidenceSyntheticRuntimePressure::ReplayVersusHistoryMasqueradeRejection,
            FoundationalBoundaryEvidenceCompileFailBoundary::LineageContinuityStrengthBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/lineage/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_evidence/lineage/replay_derived_lineage_cannot_satisfy_attested_receipt_api.rs"),
        ),
        FoundationalBoundaryEvidenceCertifiedSurfaceEvidence::new(
            FoundationalBoundaryEvidenceCertifiedSurface::SupportTruthRecoveryAndDebt,
            FoundationalBoundaryEvidenceSyntheticRuntimePressure::SupportGradeOverclaimRejection,
            FoundationalBoundaryEvidenceCompileFailBoundary::SupportGradeAndBasisDisclosureBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/support/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_evidence/support/published_support_cannot_satisfy_executed_receipt_api.rs"),
        ),
        FoundationalBoundaryEvidenceCertifiedSurfaceEvidence::new(
            FoundationalBoundaryEvidenceCertifiedSurface::AttachmentMaterializationAndReadmission,
            FoundationalBoundaryEvidenceSyntheticRuntimePressure::TrustBoundaryReadmissionWORTHry,
            FoundationalBoundaryEvidenceCompileFailBoundary::AttachmentScopeAndReadmissionBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/attachments/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_evidence/attachments/readmission/unbridged_current_basis_attachment_cannot_be_readmitted.rs"),
        ),
    ]
}

pub(super) fn compile_fail_boundaries() -> Vec<FoundationalBoundaryEvidenceCompileFailBoundary> {
    vec![
        FoundationalBoundaryEvidenceCompileFailBoundary::PrimitiveNonSubstitution,
        FoundationalBoundaryEvidenceCompileFailBoundary::ProvenanceFreshnessAndArtifactBoundaries,
        FoundationalBoundaryEvidenceCompileFailBoundary::ReceiptPlanningVersusCompletedBoundarySeparation,
        FoundationalBoundaryEvidenceCompileFailBoundary::ReplayAndHistoryRecordsCannotMasquerade,
        FoundationalBoundaryEvidenceCompileFailBoundary::LineageContinuityStrengthBoundaries,
        FoundationalBoundaryEvidenceCompileFailBoundary::SupportGradeAndBasisDisclosureBoundaries,
        FoundationalBoundaryEvidenceCompileFailBoundary::AttachmentScopeAndReadmissionBoundaries,
        FoundationalBoundaryEvidenceCompileFailBoundary::BoundaryEvidenceReadinessRequiresCertifiedArtifact,
        FoundationalBoundaryEvidenceCompileFailBoundary::BoundaryEvidenceReadinessAuthorityCannotBeMinted,
        FoundationalBoundaryEvidenceCompileFailBoundary::GroupedStrongerLaneRequiresCertifiedReadiness,
    ]
}

pub(super) fn golden_artifacts() -> Vec<FoundationalBoundaryEvidenceGoldenArtifact> {
    vec![
        FoundationalBoundaryEvidenceGoldenArtifact::PrimitiveCategoryRoleAndLocalityMeaning,
        FoundationalBoundaryEvidenceGoldenArtifact::ProvenanceLayeringAndFreshnessMeaning,
        FoundationalBoundaryEvidenceGoldenArtifact::ReceiptExecutionAndCloseoutMeaning,
        FoundationalBoundaryEvidenceGoldenArtifact::LineageContinuityPromotionAndPartialityMeaning,
        FoundationalBoundaryEvidenceGoldenArtifact::SupportTruthRecoveryAndResidualDebtMeaning,
        FoundationalBoundaryEvidenceGoldenArtifact::AttachmentCanonicalDigestAndReadmissionMeaning,
    ]
}

pub(super) fn property_seeds() -> Vec<FoundationalBoundaryEvidencePropertySeed> {
    vec![
        FoundationalBoundaryEvidencePropertySeed::PrimitiveDefinitionOrdering,
        FoundationalBoundaryEvidencePropertySeed::ProvenanceLayerAndSupportContextOrdering,
        FoundationalBoundaryEvidencePropertySeed::PlanningExecutedAndCloseoutStrength,
        FoundationalBoundaryEvidencePropertySeed::ReplayHistoryAndPromotionStrength,
        FoundationalBoundaryEvidencePropertySeed::MixedAttachmentCanonicalAndDigestParity,
    ]
}

pub(super) fn property_seed_evidence() -> Vec<FoundationalBoundaryEvidencePropertySeedEvidence> {
    vec![
        FoundationalBoundaryEvidencePropertySeedEvidence::new(
            FoundationalBoundaryEvidencePropertySeed::PrimitiveDefinitionOrdering,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/primitives/mod.rs"),
            "definition order remains blind-consumer stable across category and locality rows",
            FoundationalBoundaryEvidenceHarnessExpansionPoint::GroupedPublicSurfaceLane,
        ),
        FoundationalBoundaryEvidencePropertySeedEvidence::new(
            FoundationalBoundaryEvidencePropertySeed::ProvenanceLayerAndSupportContextOrdering,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/provenance/mod.rs"),
            "independent producer ordering cannot change canonical provenance meaning",
            FoundationalBoundaryEvidenceHarnessExpansionPoint::ReplayHistoryMasqueradeMatrix,
        ),
        FoundationalBoundaryEvidencePropertySeedEvidence::new(
            FoundationalBoundaryEvidencePropertySeed::PlanningExecutedAndCloseoutStrength,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/receipts/mod.rs"),
            "planning and closeout receipts remain weaker than executed receipts",
            FoundationalBoundaryEvidenceHarnessExpansionPoint::TrustBoundaryReadmissionParityMatrix,
        ),
        FoundationalBoundaryEvidencePropertySeedEvidence::new(
            FoundationalBoundaryEvidencePropertySeed::ReplayHistoryAndPromotionStrength,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/lineage/mod.rs"),
            "replay-derived and denied-promotion results stay visibly weaker than attested continuity",
            FoundationalBoundaryEvidenceHarnessExpansionPoint::ReplayHistoryMasqueradeMatrix,
        ),
        FoundationalBoundaryEvidencePropertySeedEvidence::new(
            FoundationalBoundaryEvidencePropertySeed::MixedAttachmentCanonicalAndDigestParity,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/attachments/mod.rs"),
            "mixed-family attachment bundles stay canonical and digest-stable across independent orderings",
            FoundationalBoundaryEvidenceHarnessExpansionPoint::MixedAttachmentCanonicalDigestParityMatrix,
        ),
    ]
}

pub(super) fn harness_expansion_points() -> Vec<FoundationalBoundaryEvidenceHarnessExpansionPoint> {
    vec![
        FoundationalBoundaryEvidenceHarnessExpansionPoint::ReplayHistoryMasqueradeMatrix,
        FoundationalBoundaryEvidenceHarnessExpansionPoint::RecoveryAndDegradedOperationMatrix,
        FoundationalBoundaryEvidenceHarnessExpansionPoint::MixedAttachmentCanonicalDigestParityMatrix,
        FoundationalBoundaryEvidenceHarnessExpansionPoint::TrustBoundaryReadmissionParityMatrix,
        FoundationalBoundaryEvidenceHarnessExpansionPoint::GroupedPublicSurfaceLane,
    ]
}

pub(super) fn runtime_assumptions() -> Vec<FoundationalBoundaryEvidenceRuntimeAssumption> {
    vec![
        FoundationalBoundaryEvidenceRuntimeAssumption::WORTHProofAuthorityLaneRemainsAvailable,
        FoundationalBoundaryEvidenceRuntimeAssumption::BoundaryArtifactAndDiagnosticMeaningRemainCertifiedDependencies,
        FoundationalBoundaryEvidenceRuntimeAssumption::CanonicalizationLawRemainsAuthorityForAttachmentParticipation,
        FoundationalBoundaryEvidenceRuntimeAssumption::ReadmissionRemainsExplicitAcrossTrustBoundaries,
    ]
}

pub(super) fn runtime_non_assumptions() -> Vec<FoundationalBoundaryEvidenceRuntimeNonAssumption> {
    vec![
        FoundationalBoundaryEvidenceRuntimeNonAssumption::RuntimeSpecificHistoryStoreLayoutOwnedHere,
        FoundationalBoundaryEvidenceRuntimeNonAssumption::ReplayDerivationUpgradesToAttestedContinuity,
        FoundationalBoundaryEvidenceRuntimeNonAssumption::SupportTruthUpgradesToAuthorityWithoutBridge,
        FoundationalBoundaryEvidenceRuntimeNonAssumption::CrossBoundaryAttachmentBundlesRemainCurrentWithoutReadmission,
    ]
}

pub(super) fn residual_debt() -> Vec<FoundationalBoundaryEvidenceResidualDebt> {
    vec![
        FoundationalBoundaryEvidenceResidualDebt::AdoptingRuntimeParityDeferred,
        FoundationalBoundaryEvidenceResidualDebt::RuntimeSpecificHistoryAndJournalTaxonomiesDeferred,
        FoundationalBoundaryEvidenceResidualDebt::RealRuntimeSupportBundlePersistenceDeferred,
    ]
}

pub(super) fn phase_gates() -> Vec<FoundationalBoundaryEvidencePhaseGateEvidence> {
    vec![
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::PrimitiveCategoryAndRoleLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/primitives/mod.rs"),
        ),
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::ProvenanceLayeringAndFreshnessLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/provenance/mod.rs"),
        ),
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::ReceiptFamilyAndCloseoutTruth,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/receipts/mod.rs"),
        ),
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::LineageContinuityAndDivergence,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/lineage/mod.rs"),
        ),
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::SupportTruthRecoveryAndDegradedOperation,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/support/mod.rs"),
        ),
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::AttachmentMaterializationAndReadmission,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/attachments/mod.rs"),
        ),
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::ProductionReadiness,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/readiness/mod.rs"),
        ),
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::FeatureDocsAndCrateDocIntegration,
            "docs/lineage-provenance-receipts-and-support-truth/README.md",
        ),
        FoundationalBoundaryEvidencePhaseGateEvidence::new(
            FoundationalBoundaryEvidenceMilestone7PhaseGate::FeatureDocWriterCloseoutAndRegistration,
            "docs/README.md",
        ),
    ]
}

pub(super) fn public_surface_inventory() -> Vec<BoundaryEvidencePublicSurfaceEntry> {
    boundary_evidence_public_surface_inventory().to_vec()
}

pub(super) fn documentation_surface_inventory() -> Vec<&'static str> {
    vec![
        "docs/README.md",
        "docs/lineage-provenance-receipts-and-support-truth/README.md",
        "docs/lineage-provenance-receipts-and-support-truth/primitive-categories-locality-and-role-postures.md",
        "docs/lineage-provenance-receipts-and-support-truth/provenance-layering-and-freshness.md",
        "docs/lineage-provenance-receipts-and-support-truth/receipts-and-closeout-truth.md",
        "docs/lineage-provenance-receipts-and-support-truth/lineage-continuity-divergence-and-promotion.md",
        "docs/lineage-provenance-receipts-and-support-truth/support-truth-recovery-and-degraded-operation.md",
        "docs/lineage-provenance-receipts-and-support-truth/attachment-materialization-canonical-participation-and-readmission.md",
        "docs/lineage-provenance-receipts-and-support-truth/grouped-public-lanes-and-stronger-readiness.md",
        "docs/lineage-provenance-receipts-and-support-truth/boundary-evidence-production-readiness.md",
    ]
}

pub(super) const fn public_surface_evidence_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-evidence/tests/worth_court_foundational_certification_boundary_evidence/foundational/certification/boundary_evidence/grouped_surface/mod.rs")
}

pub(super) const fn public_surface_compile_fail_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/boundary_evidence/grouped_surface/readiness_report_cannot_enter_grouped_stronger_lane.rs")
}
