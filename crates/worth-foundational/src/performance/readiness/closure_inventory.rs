use super::vocabulary::{
    FoundationalPerformancePublicSurfaceDocumentationCoverage, FoundationalPerformanceResidualDebt,
    FoundationalPerformanceRuntimeAdoptionPressure,
    FoundationalPerformanceRuntimeAdoptionPressureEvidence,
    FoundationalPerformanceRuntimeAssumption, FoundationalPerformanceRuntimeNonAssumption,
    FoundationalPerformanceWORTHProofApi, FoundationalPerformanceWORTHProofForbiddenSurface,
    FoundationalPerformanceWORTHProofSurface,
};
use crate::performance_api::{
    performance_public_surface_inventory, FoundationalPerformancePublicSurfaceEntry,
};

pub(super) fn worth_proof_required_surfaces() -> Vec<FoundationalPerformanceWORTHProofSurface> {
    vec![
        FoundationalPerformanceWORTHProofSurface::ProductionReadinessCertificationArtifact,
        FoundationalPerformanceWORTHProofSurface::AuthorityWitness,
        FoundationalPerformanceWORTHProofSurface::ProofFromAuthorityWitness,
        FoundationalPerformanceWORTHProofSurface::ArtifactWithProofsAndCurrentBasis,
    ]
}

pub(super) fn worth_proof_api_appendix() -> Vec<FoundationalPerformanceWORTHProofApi> {
    vec![
        FoundationalPerformanceWORTHProofApi::AuthorityWitnessFromAuthorityMarker,
        FoundationalPerformanceWORTHProofApi::ProofFromAuthorityWitness,
        FoundationalPerformanceWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
    ]
}

pub(super) fn worth_proof_forbidden_surfaces(
) -> Vec<FoundationalPerformanceWORTHProofForbiddenSurface> {
    vec![
        FoundationalPerformanceWORTHProofForbiddenSurface::PlainPerformanceVocabulary,
        FoundationalPerformanceWORTHProofForbiddenSurface::PlainPerformanceLowerLaneArtifacts,
        FoundationalPerformanceWORTHProofForbiddenSurface::PlainPerformanceReportPlanningVocabulary,
    ]
}

pub(super) fn runtime_assumptions() -> Vec<FoundationalPerformanceRuntimeAssumption> {
    vec![
        FoundationalPerformanceRuntimeAssumption::WORTHProofAuthorityLaneRemainsAvailable,
        FoundationalPerformanceRuntimeAssumption::ProfileLawRemainsAuthorityForReportElision,
        FoundationalPerformanceRuntimeAssumption::PhaseEvidencePathsRemainOwnedWithinFoundational,
    ]
}

pub(super) fn runtime_non_assumptions() -> Vec<FoundationalPerformanceRuntimeNonAssumption> {
    vec![FoundationalPerformanceRuntimeNonAssumption::WorkspaceWideTelemetryEngineIsOwnedHere]
}

pub(super) fn residual_debt() -> Vec<FoundationalPerformanceResidualDebt> {
    vec![]
}

pub(super) fn runtime_adoption_pressures() -> Vec<FoundationalPerformanceRuntimeAdoptionPressure> {
    vec![
        FoundationalPerformanceRuntimeAdoptionPressure::CrossCrateMeaningParityMatrix,
        FoundationalPerformanceRuntimeAdoptionPressure::CertifiedBundleSourceCompatibilityMatrix,
    ]
}

pub(super) fn runtime_adoption_pressure_evidence(
) -> Vec<FoundationalPerformanceRuntimeAdoptionPressureEvidence> {
    vec![
        FoundationalPerformanceRuntimeAdoptionPressureEvidence::new(
            FoundationalPerformanceRuntimeAdoptionPressure::CrossCrateMeaningParityMatrix,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/runtime_parity/mod.rs"),
        ),
        FoundationalPerformanceRuntimeAdoptionPressureEvidence::new(
            FoundationalPerformanceRuntimeAdoptionPressure::CertifiedBundleSourceCompatibilityMatrix,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/runtime_parity/mod.rs"),
        ),
    ]
}

pub(super) fn public_surface_inventory() -> Vec<FoundationalPerformancePublicSurfaceEntry> {
    performance_public_surface_inventory().to_vec()
}

pub(super) fn documentation_surface_inventory() -> Vec<&'static str> {
    vec![
        "docs/README.md",
        "docs/performance/README.md",
        "docs/performance/common-performance-claims-and-layout-intent.md",
        "docs/performance/policy-admission-receipts.md",
        "docs/performance/canonical-bundles-and-comparison.md",
        "docs/performance/counter-backed-performance-receipts.md",
        "docs/performance/performance-report-planning-and-materialization.md",
        "docs/performance/certified-and-readmitted-performance-bundles.md",
        "docs/performance/grouped-public-lanes-and-stronger-readiness.md",
        "docs/performance/performance-production-readiness.md",
    ]
}

pub(super) fn public_surface_documentation_coverage(
) -> Vec<FoundationalPerformancePublicSurfaceDocumentationCoverage> {
    vec![
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::common_path",
            "docs/performance/common-performance-claims-and-layout-intent.md",
        ),
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::lower_lane::basis",
            "docs/performance/canonical-bundles-and-comparison.md",
        ),
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::lower_lane::policy",
            "docs/performance/policy-admission-receipts.md",
        ),
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::lower_lane::receipts",
            "docs/performance/counter-backed-performance-receipts.md",
        ),
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::lower_lane::reports",
            "docs/performance/performance-report-planning-and-materialization.md",
        ),
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::lower_lane",
            "docs/performance/grouped-public-lanes-and-stronger-readiness.md",
        ),
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::stronger_lane",
            "docs/performance/grouped-public-lanes-and-stronger-readiness.md",
        ),
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::stronger_lane::certified",
            "docs/performance/certified-and-readmitted-performance-bundles.md",
        ),
        FoundationalPerformancePublicSurfaceDocumentationCoverage::new(
            "worth_foundational::performance_api::stronger_lane::readiness",
            "docs/performance/performance-production-readiness.md",
        ),
    ]
}

pub(super) const fn public_surface_evidence_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-foundational-certification-boundary-artifacts/tests/worth_court_foundational_certification_boundary_artifacts/foundational/certification/performance/grouped_surface/mod.rs")
}

pub(super) const fn public_surface_compile_fail_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-foundational/assets/ui/performance/grouped_surface_stronger_lane/plain_counter_backed_receipt_cannot_enter_grouped_stronger_lane_certified_api.rs")
}
