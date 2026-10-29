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
            "tests/worth_foundational/fast/certification/performance/primitives.rs",
            "tests/ui/performance/family_boundaries/evidence_strength_api_rejects_execution_temperature.rs",
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::ClaimBoundaryAndEvidenceStrengthLaw,
            FoundationalPerformanceSyntheticRuntimePressure::ClaimStrengthAndLaneCollapseRejection,
            FoundationalPerformanceCompileFailBoundary::ClaimLaneBoundaries,
            "tests/worth_foundational/fast/certification/performance/claims.rs",
            "tests/ui/performance/claims/support_claim_cannot_satisfy_authoritative_claim_api.rs",
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::LayoutIntentAndRepresentationFreedom,
            FoundationalPerformanceSyntheticRuntimePressure::RepresentationEquivalenceOverclaimRejection,
            FoundationalPerformanceCompileFailBoundary::LayoutAttachmentBoundaries,
            "tests/worth_foundational/fast/certification/performance/layouts.rs",
            "tests/ui/performance/layouts/plain_claim_cannot_satisfy_layout_annotated_claim_api.rs",
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::PolicyAdmissionAndBudgetLaw,
            FoundationalPerformanceSyntheticRuntimePressure::PreExecutionMasqueradeRejection,
            FoundationalPerformanceCompileFailBoundary::PolicyPreExecutionBoundaries,
            "tests/worth_foundational/fast/certification/performance/policy.rs",
            "tests/ui/performance/policy/plain_policy_claim_cannot_satisfy_policy_receipt_api.rs",
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::CanonicalBundleAndCounterReceiptLaw,
            FoundationalPerformanceSyntheticRuntimePressure::CanonicalCounterLoweringRejection,
            FoundationalPerformanceCompileFailBoundary::BundleAndCounterReceiptLoweringBoundaries,
            "tests/worth_foundational/fast/certification/performance/basis.rs",
            "tests/ui/performance/basis/plain_claim_cannot_satisfy_bundle_api.rs",
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::ReportAttachmentAndMaterializationLaw,
            FoundationalPerformanceSyntheticRuntimePressure::HiddenSupportExpansionRejection,
            FoundationalPerformanceCompileFailBoundary::ReportMaterializationBoundaries,
            "tests/worth_foundational/fast/certification/performance/reports.rs",
            "tests/ui/performance/reports/attached_bundle_cannot_satisfy_materialized_report_api.rs",
        ),
        FoundationalPerformanceCertifiedSurfaceEvidence::new(
            FoundationalPerformanceCertifiedSurface::CertifiedBundleAndReadmissionLaw,
            FoundationalPerformanceSyntheticRuntimePressure::CertifiedProofLaneBoundary,
            FoundationalPerformanceCompileFailBoundary::CertifiedBundleAndReadmissionProofLane,
            "tests/worth_foundational/fast/certification/performance/certified.rs",
            "tests/ui/performance/certified/plain_counter_backed_receipt_cannot_satisfy_certified_bundle_api.rs",
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
            "tests/worth_foundational/fast/certification/performance/primitives.rs",
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::ClaimBoundaryAndEvidenceStrengthLaw,
            "tests/worth_foundational/fast/certification/performance/claims.rs",
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::LayoutIntentAccessAndAllocationLaw,
            "tests/worth_foundational/fast/certification/performance/layouts.rs",
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::RuntimePolicyBudgetAndFallbackLaw,
            "tests/worth_foundational/fast/certification/performance/policy.rs",
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::CanonicalBasisCounterAndComparisonLaw,
            "tests/worth_foundational/fast/certification/performance/basis.rs",
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::AttachmentMaterializationAndBundleLaw,
            "tests/worth_foundational/fast/certification/performance/reports.rs",
        ),
        FoundationalPerformancePhaseGateEvidence::new(
            FoundationalPerformanceMilestone8PhaseGate::ProductionReadiness,
            "tests/worth_foundational/fast/certification/performance/readiness.rs",
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
