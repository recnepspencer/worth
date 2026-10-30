use super::vocabulary::{
    FoundationalPerformanceCertifiedSurface, FoundationalPerformanceCertifiedSurfaceEvidence,
    FoundationalPerformanceCompileFailBoundary, FoundationalPerformanceHarnessExpansionPoint,
    FoundationalPerformanceMilestone8PhaseGate, FoundationalPerformancePhaseGateEvidence,
    FoundationalPerformanceSyntheticRuntimePressure,
};

pub(super) fn certified_surfaces() -> Vec<FoundationalPerformanceCertifiedSurface> {
    vec![
        FoundationalPerformanceCertifiedSurface::PrimitiveAndCategoryLaw,
        FoundationalPerformanceCertifiedSurface::ClaimBoundaryAndEvidenceStrengthLaw,
        FoundationalPerformanceCertifiedSurface::LayoutIntentAndRepresentationFreedom,
        FoundationalPerformanceCertifiedSurface::PolicyAdmissionAndBudgetLaw,
        FoundationalPerformanceCertifiedSurface::CanonicalBundleAndCounterReceiptLaw,
        FoundationalPerformanceCertifiedSurface::ReportAttachmentAndMaterializationLaw,
        FoundationalPerformanceCertifiedSurface::CertifiedBundleAndReadmissionLaw,
    ]
}

pub(super) fn synthetic_pressures() -> Vec<FoundationalPerformanceSyntheticRuntimePressure> {
    vec![
        FoundationalPerformanceSyntheticRuntimePressure::PrimitiveFamilyNonSubstitution,
        FoundationalPerformanceSyntheticRuntimePressure::ClaimStrengthAndLaneCollapseRejection,
        FoundationalPerformanceSyntheticRuntimePressure::RepresentationEquivalenceOverclaimRejection,
        FoundationalPerformanceSyntheticRuntimePressure::PreExecutionMasqueradeRejection,
        FoundationalPerformanceSyntheticRuntimePressure::CanonicalCounterLoweringRejection,
        FoundationalPerformanceSyntheticRuntimePressure::HiddenSupportExpansionRejection,
        FoundationalPerformanceSyntheticRuntimePressure::CertifiedProofLaneBoundary,
        FoundationalPerformanceSyntheticRuntimePressure::GroupedStrongerLaneBoundary,
    ]
}

pub(super) fn certified_surface_evidence() -> Vec<FoundationalPerformanceCertifiedSurfaceEvidence> {
    vec![
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::PrimitiveAndCategoryLaw,
            FoundationalPerformanceSyntheticRuntimePressure::PrimitiveFamilyNonSubstitution,
            FoundationalPerformanceCompileFailBoundary::PrimitiveFamiliesAndCommonPathBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/primitives/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/performance/family_boundaries/evidence_strength_api_rejects_execution_temperature.rs"),
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::ClaimBoundaryAndEvidenceStrengthLaw,
            FoundationalPerformanceSyntheticRuntimePressure::ClaimStrengthAndLaneCollapseRejection,
            FoundationalPerformanceCompileFailBoundary::ClaimLaneBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/claims/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/performance/claims/support_claim_cannot_satisfy_authoritative_claim_api.rs"),
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::LayoutIntentAndRepresentationFreedom,
            FoundationalPerformanceSyntheticRuntimePressure::RepresentationEquivalenceOverclaimRejection,
            FoundationalPerformanceCompileFailBoundary::LayoutAttachmentBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/layouts/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/performance/layouts/plain_claim_cannot_satisfy_layout_annotated_claim_api.rs"),
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::PolicyAdmissionAndBudgetLaw,
            FoundationalPerformanceSyntheticRuntimePressure::PreExecutionMasqueradeRejection,
            FoundationalPerformanceCompileFailBoundary::PolicyPreExecutionBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/policy/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/performance/policy/plain_policy_claim_cannot_satisfy_policy_receipt_api.rs"),
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::CanonicalBundleAndCounterReceiptLaw,
            FoundationalPerformanceSyntheticRuntimePressure::CanonicalCounterLoweringRejection,
            FoundationalPerformanceCompileFailBoundary::BundleAndCounterReceiptLoweringBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/basis/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/performance/basis/plain_claim_cannot_satisfy_bundle_api.rs"),
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::ReportAttachmentAndMaterializationLaw,
            FoundationalPerformanceSyntheticRuntimePressure::HiddenSupportExpansionRejection,
            FoundationalPerformanceCompileFailBoundary::ReportMaterializationBoundaries,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/reports/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/performance/reports/attached_bundle_cannot_satisfy_materialized_report_api.rs"),
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::CertifiedBundleAndReadmissionLaw,
            FoundationalPerformanceSyntheticRuntimePressure::CertifiedProofLaneBoundary,
            FoundationalPerformanceCompileFailBoundary::CertifiedBundleAndReadmissionProofLane,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/certified/mod.rs"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/performance/certified/plain_counter_backed_receipt_cannot_satisfy_certified_bundle_api.rs"),
        ),
    ]
}

pub(super) fn compile_fail_boundaries() -> Vec<FoundationalPerformanceCompileFailBoundary> {
    vec![
        FoundationalPerformanceCompileFailBoundary::PrimitiveFamiliesAndCommonPathBoundaries,
        FoundationalPerformanceCompileFailBoundary::ClaimLaneBoundaries,
        FoundationalPerformanceCompileFailBoundary::LayoutAttachmentBoundaries,
        FoundationalPerformanceCompileFailBoundary::PolicyPreExecutionBoundaries,
        FoundationalPerformanceCompileFailBoundary::BundleAndCounterReceiptLoweringBoundaries,
        FoundationalPerformanceCompileFailBoundary::ReportMaterializationBoundaries,
        FoundationalPerformanceCompileFailBoundary::CertifiedBundleAndReadmissionProofLane,
        FoundationalPerformanceCompileFailBoundary::PerformanceReadinessRequiresCertifiedArtifact,
        FoundationalPerformanceCompileFailBoundary::PerformanceReadinessAuthorityCannotBeMinted,
        FoundationalPerformanceCompileFailBoundary::GroupedStrongerLaneRequiresCertifiedReadiness,
    ]
}

pub(super) fn phase_gates() -> Vec<FoundationalPerformancePhaseGateEvidence> {
    vec![
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::PrimitiveAndCategoryLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/primitives/mod.rs"),
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::ClaimBoundaryAndEvidenceStrengthLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/claims/mod.rs"),
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::LayoutIntentAccessAndAllocationLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/layouts/mod.rs"),
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::RuntimePolicyBudgetAndFallbackLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/policy/mod.rs"),
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::CanonicalBasisCounterAndComparisonLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/basis/mod.rs"),
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::AttachmentMaterializationAndBundleLaw,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/reports/mod.rs"),
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::ProductionReadiness,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/readiness/mod.rs"),
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::FeatureDocsCrateDocIntegrationAndPublicationClosure,
            "docs/performance/README.md",
        ),
    ]
}

pub(super) fn harness_expansion_points() -> Vec<FoundationalPerformanceHarnessExpansionPoint> {
    vec![FoundationalPerformanceHarnessExpansionPoint::PolicyUnavailableSectionMatrix]
}
