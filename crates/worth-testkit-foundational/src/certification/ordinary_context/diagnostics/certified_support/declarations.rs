pub use worth_proof::TransitionOutcome;
pub use {
    worth_foundational::admit_current_basis_boundary_artifact,
    worth_foundational::admit_current_basis_commit_receipt,
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::foundational_boundary_current_basis_authority,
    worth_foundational::foundational_commit_receipt_issuance,
    worth_foundational::foundational_diagnostic_certified_attachment_authority,
    worth_foundational::foundational_profile_progression_authority,
    worth_foundational::foundational_transition_current_basis_authority,
    worth_foundational::materialize_admitted_foundational_profile,
    worth_foundational::materialize_descriptive_boundary_surface,
    worth_foundational::request_foundational_profile_set, worth_foundational::BoundaryHandle,
    worth_foundational::CanonicalizationRuleVersion,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile,
    worth_foundational::CurrentBasisBoundaryArtifact,
    worth_foundational::CurrentBasisCommitReceiptArtifact,
    worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalBoundaryArtifactSurface,
    worth_foundational::FoundationalBoundaryMaterializationSeam,
    worth_foundational::FoundationalBoundaryMaterializationSource,
    worth_foundational::FoundationalCommitId,
    worth_foundational::FoundationalCommitReceiptIdentity,
    worth_foundational::FoundationalDiagnosticAssemblyDebt,
    worth_foundational::FoundationalDiagnosticAssemblyDebtClass,
    worth_foundational::FoundationalDiagnosticCertifiedCoverageDenial,
    worth_foundational::FoundationalDiagnosticCoverageFamilyStatus,
    worth_foundational::FoundationalDiagnosticCoverageMatrix,
    worth_foundational::FoundationalDiagnosticExplanationBundle,
    worth_foundational::FoundationalDiagnosticGapClass,
    worth_foundational::FoundationalDiagnosticGapClosurePosture,
    worth_foundational::FoundationalDiagnosticGapTarget,
    worth_foundational::FoundationalDiagnosticNamedGap,
    worth_foundational::FoundationalDiagnosticPartiality,
    worth_foundational::FoundationalDiagnosticSupportClaimStrength,
    worth_foundational::FoundationalDiagnosticSupportReport,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

pub use {
    crate::certification::ordinary_context::diagnostics::materialization_support::explanation_input,
    crate::certification::ordinary_context::diagnostics::materialization_support::subject,
    crate::certification::ordinary_context::diagnostics::materialization_support::support_input,
};
pub use {
    crate::certification::ordinary_context::transitions::fixtures::committed::accepted_verdict,
    crate::certification::ordinary_context::transitions::fixtures::committed::committed_authority,
    crate::certification::ordinary_context::transitions::fixtures::committed::ordinary_commit_input,
};

pub fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name).expect("valid version")
}

pub fn production_certified_profile() -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
        support_posture: worth_foundational::SupportPostureProfile::CertificationReady,
        compatibility_posture: worth_foundational::CompatibilityPostureProfile::NativeOnly,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
        certification_posture: worth_foundational::CertificationPostureProfile::ProductionCertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("coherent production-certified profile")
}

pub fn certified_support_report_complete() -> FoundationalDiagnosticSupportReport {
    worth_foundational::materialize_diagnostic_support_report(
        support_input(
            worth_foundational::FoundationalDiagnosticSupportClaimStrength::CertifiedSupportReady,
            worth_foundational::FoundationalDiagnosticPartiality::Complete,
            vec![worth_foundational::FoundationalDiagnosticAssemblyDebt::new(
                worth_foundational::FoundationalDiagnosticAssemblyDebtClass::RowScanFallback,
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
        explanation_input(
            worth_foundational::FoundationalDiagnosticPartiality::PartialWithNamedGaps(vec![
                worth_foundational::FoundationalDiagnosticNamedGap::new(
                    worth_foundational::FoundationalDiagnosticGapClass::CoverageOmission,
                    worth_foundational::FoundationalDiagnosticGapTarget::Subject(subject()),
                    worth_foundational::FoundationalDiagnosticGapClosurePosture::Deferred,
                ),
            ]),
        ),
        production_certified_profile(),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("partial explanation bundle")
}

pub fn hostile_support_coverage_matrix() -> FoundationalDiagnosticCoverageMatrix {
    worth_foundational::FoundationalDiagnosticCoverageMatrix::new(
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::HostileRowsPresent {
            row_count: 2,
        },
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
    )
}

pub fn partial_explanation_coverage_matrix(
    gap: FoundationalDiagnosticNamedGap,
) -> FoundationalDiagnosticCoverageMatrix {
    worth_foundational::FoundationalDiagnosticCoverageMatrix::new(
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::PartialWithNamedGap(gap),
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
    )
}

pub fn happy_path_denied_matrix() -> FoundationalDiagnosticCoverageMatrix {
    worth_foundational::FoundationalDiagnosticCoverageMatrix::new(
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::Denied(
            worth_foundational::FoundationalDiagnosticCertifiedCoverageDenial::HappyPathOnlyDenied,
        ),
        worth_foundational::FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
    )
}

pub fn current_basis_receipt_source() -> CurrentBasisCommitReceiptArtifact {
    let receipt = accepted_verdict("diagnostic-certification")
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority")
        .issue_receipt(
            worth_foundational::FoundationalCommitReceiptIdentity::new(
                worth_foundational::BoundaryHandle::new(701),
            ),
            worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(
                700,
            )),
            foundational_commit_receipt_issuance(),
        )
        .expect("receipt");

    match admit_current_basis_commit_receipt(
        version("m6.phase5.receipt"),
        receipt,
        foundational_transition_current_basis_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(artifact) => artifact,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("expected current-basis receipt source, denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(_) => {
            panic!("expected current-basis receipt source, deferred")
        }
        worth_proof::TransitionOutcome::Stale(_) => {
            panic!("expected current-basis receipt source, stale")
        }
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("expected current-basis receipt source, rebind required")
        }
        worth_proof::TransitionOutcome::Failed(_) => {
            panic!("expected current-basis receipt source, failed")
        }
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
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("expected admitted profile for boundary source, denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(_) => {
            panic!("expected admitted profile for boundary source, deferred")
        }
        worth_proof::TransitionOutcome::Stale(_) => {
            panic!("expected admitted profile for boundary source, stale")
        }
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("expected admitted profile for boundary source, rebind required")
        }
        worth_proof::TransitionOutcome::Failed(_) => {
            panic!("expected admitted profile for boundary source, failed")
        }
    };
    let materialized_profile = match materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(materialized) => *materialized.payload(),
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("expected materialized profile for boundary source, denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(_) => {
            panic!("expected materialized profile for boundary source, deferred")
        }
        worth_proof::TransitionOutcome::Stale(_) => {
            panic!("expected materialized profile for boundary source, stale")
        }
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("expected materialized profile for boundary source, rebind required")
        }
        worth_proof::TransitionOutcome::Failed(_) => {
            panic!("expected materialized profile for boundary source, failed")
        }
    };
    let materialized_boundary = materialize_descriptive_boundary_surface(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![9_u8, 8, 7], 2),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        materialized_profile,
    )
    .expect("materialized boundary artifact");

    match admit_current_basis_boundary_artifact(
        version("m6.phase5.boundary"),
        materialized_boundary,
        foundational_boundary_current_basis_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(artifact) => artifact,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("expected current-basis boundary artifact source, denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(_) => {
            panic!("expected current-basis boundary artifact source, deferred")
        }
        worth_proof::TransitionOutcome::Stale(_) => {
            panic!("expected current-basis boundary artifact source, stale")
        }
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("expected current-basis boundary artifact source, rebind required")
        }
        worth_proof::TransitionOutcome::Failed(_) => {
            panic!("expected current-basis boundary artifact source, failed")
        }
    }
}
