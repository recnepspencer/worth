use worth_foundational::{
    admit_current_basis_boundary_artifact, admit_current_basis_commit_receipt,
    claim_derived_projection_boundary_surface, foundational_boundary_current_basis_authority,
    foundational_commit_receipt_issuance, foundational_diagnostic_certified_attachment_authority,
    foundational_profile_progression_authority, foundational_transition_current_basis_authority,
    materialize_admitted_foundational_profile, materialize_descriptive_boundary_surface,
    request_foundational_profile_set, BoundaryHandle, CanonicalizationRuleVersion,
    CertificationPostureProfile, CompatibilityPostureProfile, CurrentBasisBoundaryArtifact,
    CurrentBasisCommitReceiptArtifact, DiagnosticRichnessProfile,
    FoundationalBoundaryArtifactSurface, FoundationalBoundaryMaterializationSeam,
    FoundationalBoundaryMaterializationSource, FoundationalCommitId,
    FoundationalCommitReceiptIdentity, FoundationalDiagnosticAssemblyDebt,
    FoundationalDiagnosticAssemblyDebtClass, FoundationalDiagnosticCertifiedCoverageDenial,
    FoundationalDiagnosticCoverageFamilyStatus, FoundationalDiagnosticCoverageMatrix,
    FoundationalDiagnosticExplanationBundle, FoundationalDiagnosticGapClass,
    FoundationalDiagnosticGapClosurePosture, FoundationalDiagnosticGapTarget,
    FoundationalDiagnosticNamedGap, FoundationalDiagnosticPartiality,
    FoundationalDiagnosticSupportClaimStrength, FoundationalDiagnosticSupportReport,
    FoundationalProfileSet, FoundationalProfileSetInput, RetentionDeliveryProfile,
    SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::materialization_support::{explanation_input, subject, support_input};
use crate::certification::transitions::fixtures::committed::{
    accepted_verdict, committed_authority, ordinary_commit_input,
};

pub fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid version")
}

pub fn production_certified_profile() -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::CertificationReady,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::ProductionCertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("coherent production-certified profile")
}

pub fn certified_support_report_complete() -> FoundationalDiagnosticSupportReport {
    worth_foundational::materialize_diagnostic_support_report(
        support_input(
            FoundationalDiagnosticSupportClaimStrength::CertifiedSupportReady,
            FoundationalDiagnosticPartiality::Complete,
            vec![FoundationalDiagnosticAssemblyDebt::new(
                FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
                2,
            )],
        ),
        production_certified_profile(),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("certified support report")
}

pub fn partial_explanation_bundle() -> FoundationalDiagnosticExplanationBundle {
    worth_foundational::materialize_diagnostic_explanation_bundle(
        explanation_input(FoundationalDiagnosticPartiality::PartialWithNamedGaps(
            vec![FoundationalDiagnosticNamedGap::new(
                FoundationalDiagnosticGapClass::CoverageOmission,
                FoundationalDiagnosticGapTarget::Subject(subject()),
                FoundationalDiagnosticGapClosurePosture::Deferred,
            )],
        )),
        production_certified_profile(),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("partial explanation bundle")
}

pub fn hostile_support_coverage_matrix() -> FoundationalDiagnosticCoverageMatrix {
    FoundationalDiagnosticCoverageMatrix::new(
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::HostileRowsPresent { row_count: 2 },
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
    )
}

pub fn partial_explanation_coverage_matrix(
    gap: FoundationalDiagnosticNamedGap,
) -> FoundationalDiagnosticCoverageMatrix {
    FoundationalDiagnosticCoverageMatrix::new(
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::PartialWithNamedGap(gap),
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
    )
}

pub fn happy_path_denied_matrix() -> FoundationalDiagnosticCoverageMatrix {
    FoundationalDiagnosticCoverageMatrix::new(
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        FoundationalDiagnosticCoverageFamilyStatus::Denied(
            FoundationalDiagnosticCertifiedCoverageDenial::HappyPathOnlyDenied,
        ),
        FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
    )
}

pub fn current_basis_receipt_source() -> CurrentBasisCommitReceiptArtifact {
    let receipt = accepted_verdict("diagnostic-certification")
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority")
        .issue_receipt(
            FoundationalCommitReceiptIdentity::new(BoundaryHandle::new(701)),
            FoundationalCommitId::new(BoundaryHandle::new(700)),
            foundational_commit_receipt_issuance(),
        )
        .expect("receipt");

    match admit_current_basis_commit_receipt(
        version("m6.phase5.receipt"),
        receipt,
        foundational_transition_current_basis_authority(),
    ) {
        TransitionOutcome::Success(artifact) => artifact,
        TransitionOutcome::Denied(denial) => {
            panic!("expected current-basis receipt source, denied: {denial:?}")
        }
        TransitionOutcome::Deferred(_) => panic!("expected current-basis receipt source, deferred"),
        TransitionOutcome::Stale(_) => panic!("expected current-basis receipt source, stale"),
        TransitionOutcome::RebindRequired(_) => {
            panic!("expected current-basis receipt source, rebind required")
        }
        TransitionOutcome::Failed(_) => panic!("expected current-basis receipt source, failed"),
    }
}

pub fn certification_authority() -> worth_proof::AuthorityWitness<
    worth_foundational::FoundationalDiagnosticCertifiedAttachmentAuthority,
> {
    foundational_diagnostic_certified_attachment_authority()
}

pub fn current_basis_boundary_artifact_source(
) -> CurrentBasisBoundaryArtifact<FoundationalBoundaryArtifactSurface<Vec<u8>>> {
    let profile = production_certified_profile();
    let requested = request_foundational_profile_set(profile);
    let admitted = match worth_foundational::admit_requested_foundational_profile(
        requested,
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        TransitionOutcome::Denied(denial) => {
            panic!("expected admitted profile for boundary source, denied: {denial:?}")
        }
        TransitionOutcome::Deferred(_) => {
            panic!("expected admitted profile for boundary source, deferred")
        }
        TransitionOutcome::Stale(_) => {
            panic!("expected admitted profile for boundary source, stale")
        }
        TransitionOutcome::RebindRequired(_) => {
            panic!("expected admitted profile for boundary source, rebind required")
        }
        TransitionOutcome::Failed(_) => {
            panic!("expected admitted profile for boundary source, failed")
        }
    };
    let materialized_profile = match materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(materialized) => *materialized.payload(),
        TransitionOutcome::Denied(denial) => {
            panic!("expected materialized profile for boundary source, denied: {denial:?}")
        }
        TransitionOutcome::Deferred(_) => {
            panic!("expected materialized profile for boundary source, deferred")
        }
        TransitionOutcome::Stale(_) => {
            panic!("expected materialized profile for boundary source, stale")
        }
        TransitionOutcome::RebindRequired(_) => {
            panic!("expected materialized profile for boundary source, rebind required")
        }
        TransitionOutcome::Failed(_) => {
            panic!("expected materialized profile for boundary source, failed")
        }
    };
    let materialized_boundary = materialize_descriptive_boundary_surface(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![9_u8, 8, 7],
            2,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        materialized_profile,
    )
    .expect("materialized boundary artifact");

    match admit_current_basis_boundary_artifact(
        version("m6.phase5.boundary"),
        materialized_boundary,
        foundational_boundary_current_basis_authority(),
    ) {
        TransitionOutcome::Success(artifact) => artifact,
        TransitionOutcome::Denied(denial) => {
            panic!("expected current-basis boundary artifact source, denied: {denial:?}")
        }
        TransitionOutcome::Deferred(_) => {
            panic!("expected current-basis boundary artifact source, deferred")
        }
        TransitionOutcome::Stale(_) => {
            panic!("expected current-basis boundary artifact source, stale")
        }
        TransitionOutcome::RebindRequired(_) => {
            panic!("expected current-basis boundary artifact source, rebind required")
        }
        TransitionOutcome::Failed(_) => {
            panic!("expected current-basis boundary artifact source, failed")
        }
    }
}
