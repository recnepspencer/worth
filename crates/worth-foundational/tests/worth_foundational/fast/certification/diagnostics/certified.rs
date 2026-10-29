use worth_foundational::{
    bridge_certified_diagnostic_bundle_trust_boundary, certify_current_basis_diagnostic_bundle,
    certify_diagnostic_bundle_with_source_basis,
    foundational_diagnostic_certified_readmission_authority,
    prepare_diagnostic_support_report_for_canonical_basis,
    readmit_certified_diagnostic_bundle_after_boundary,
    FoundationalDiagnosticCertifiedAttachmentDenial, FoundationalDiagnosticCertifiedCoverageClass,
    FoundationalDiagnosticCoverageFamilyStatus, FoundationalDiagnosticCoverageMatrix,
    FoundationalDiagnosticNamedGap,
};
use worth_proof::TransitionOutcome;

use super::certified_support::{
    certification_authority, certified_support_report_complete,
    current_basis_boundary_artifact_source, current_basis_receipt_source, happy_path_denied_matrix,
    hostile_support_coverage_matrix, partial_explanation_bundle,
    partial_explanation_coverage_matrix, version,
};

#[test]
fn certified_diagnostic_bundle_reuses_proof_lane_for_current_basis_transition_sources() {
    let version = version("m6.phase5.transition");
    let report = certified_support_report_complete();
    let rebound_basis =
        match prepare_diagnostic_support_report_for_canonical_basis(version.clone(), &report) {
            TransitionOutcome::Success(ready) => ready,
            TransitionOutcome::Denied(denial) => {
                panic!("expected canonical basis for certified support report, denied: {denial:?}")
            }
            TransitionOutcome::Deferred(_) => {
                panic!("expected canonical basis for certified support report, deferred")
            }
            TransitionOutcome::Stale(_) => {
                panic!("expected canonical basis for certified support report, stale")
            }
            TransitionOutcome::RebindRequired(_) => {
                panic!("expected canonical basis for certified support report, rebind required")
            }
            TransitionOutcome::Failed(_) => {
                panic!("expected canonical basis for certified support report, failed")
            }
        };

    let certified = match certify_current_basis_diagnostic_bundle(
        version,
        current_basis_receipt_source(),
        report,
        hostile_support_coverage_matrix(),
        certification_authority(),
    ) {
        TransitionOutcome::Success(bundle) => bundle,
        TransitionOutcome::Denied(denial) => {
            panic!("expected certified diagnostic bundle, denied: {denial:?}")
        }
        TransitionOutcome::Deferred(_) => panic!("expected certified diagnostic bundle, deferred"),
        TransitionOutcome::Stale(_) => panic!("expected certified diagnostic bundle, stale"),
        TransitionOutcome::RebindRequired(_) => {
            panic!("expected certified diagnostic bundle, rebind required")
        }
        TransitionOutcome::Failed(_) => panic!("expected certified diagnostic bundle, failed"),
    };

    assert_eq!(
        certified.coverage_class(),
        FoundationalDiagnosticCertifiedCoverageClass::HostileCoveragePresent
    );
    assert_eq!(
        certified.source_kind(),
        worth_foundational::FoundationalCertifiedDiagnosticSourceKind::CurrentBasisCommitReceipt
    );
    assert_eq!(
        certified.provenance_hook(),
        worth_foundational::FoundationalCertifiedDiagnosticProvenanceHook::TransitionEvidenceOriginAttachment
    );
    assert_eq!(
        certified.source_digest().domain(),
        worth_foundational::CanonicalBasisDomain::Transition
    );
    assert_eq!(
        certified.strong_basis().payload().domain(),
        worth_foundational::CanonicalBasisDomain::Diagnostic
    );
    assert_eq!(certified.bundle().support_rows().count(), 2);

    let readmitted = readmit_certified_diagnostic_bundle_after_boundary(
        bridge_certified_diagnostic_bundle_trust_boundary(certified),
        rebound_basis,
        foundational_diagnostic_certified_readmission_authority(),
    );
    assert_eq!(readmitted.bundle().support_rows().count(), 2);
}

#[test]
fn certified_diagnostic_bundle_preserves_partial_named_gap_coverage_honestly() {
    let source = current_basis_boundary_artifact_source();
    let bundle = partial_explanation_bundle();
    let gap = bundle.named_gaps()[0].clone();

    let certified = match certify_current_basis_diagnostic_bundle(
        version("m6.phase5.boundary"),
        source,
        bundle,
        partial_explanation_coverage_matrix(gap.clone()),
        certification_authority(),
    ) {
        TransitionOutcome::Success(bundle) => bundle,
        TransitionOutcome::Denied(denial) => {
            panic!("expected partial certified bundle, denied: {denial:?}")
        }
        TransitionOutcome::Deferred(_) => {
            panic!("expected partial certified bundle, deferred")
        }
        TransitionOutcome::Stale(_) => panic!("expected partial certified bundle, stale"),
        TransitionOutcome::RebindRequired(_) => {
            panic!("expected partial certified bundle, rebind required")
        }
        TransitionOutcome::Failed(_) => panic!("expected partial certified bundle, failed"),
    };

    assert_eq!(
        certified.coverage_class(),
        FoundationalDiagnosticCertifiedCoverageClass::PartialWithNamedGaps
    );
    assert_eq!(
        certified.source_kind(),
        worth_foundational::FoundationalCertifiedDiagnosticSourceKind::CurrentBasisBoundaryArtifact
    );
    assert_eq!(
        certified.provenance_hook(),
        worth_foundational::FoundationalCertifiedDiagnosticProvenanceHook::BoundaryArtifactEvidenceOriginAttachment
    );
    assert_eq!(certified.bundle().named_gaps(), &[gap]);
}

#[test]
fn certified_diagnostic_bundle_denies_missing_source_digest_and_happy_path_only_coverage() {
    let report = certified_support_report_complete();

    assert!(matches!(
        certify_diagnostic_bundle_with_source_basis(
            version("m6.phase5.missing_source"),
            worth_foundational::FoundationalCertifiedDiagnosticSourceKind::CurrentBasisCommitReceipt,
            worth_foundational::FoundationalCertifiedDiagnosticProvenanceHook::TransitionEvidenceOriginAttachment,
            None,
            "synthetic-source",
            report,
            hostile_support_coverage_matrix(),
            certification_authority(),
        ),
        TransitionOutcome::Denied(
            FoundationalDiagnosticCertifiedAttachmentDenial::MissingSourceDigest
        )
    ));

    assert!(matches!(
        certify_current_basis_diagnostic_bundle(
            version("m6.phase5.happy_path_only"),
            current_basis_receipt_source(),
            certified_support_report_complete(),
            happy_path_denied_matrix(),
            certification_authority(),
        ),
        TransitionOutcome::Denied(
            FoundationalDiagnosticCertifiedAttachmentDenial::HappyPathOnlyDenied
        )
    ));
}

#[test]
fn certified_diagnostic_bundle_denies_fake_family_coverage_and_unbound_named_gaps() {
    assert!(matches!(
        certify_current_basis_diagnostic_bundle(
            version("m6.phase5.fake_family"),
            current_basis_receipt_source(),
            certified_support_report_complete(),
            FoundationalDiagnosticCoverageMatrix::new(
                FoundationalDiagnosticCoverageFamilyStatus::HostileRowsPresent { row_count: 1 },
                FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
                FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
                FoundationalDiagnosticCoverageFamilyStatus::HostileRowsPresent { row_count: 2 },
                FoundationalDiagnosticCoverageFamilyStatus::AbsentFromBundle,
            ),
            certification_authority(),
        ),
        TransitionOutcome::Denied(
            FoundationalDiagnosticCertifiedAttachmentDenial::CoveredFamilyCannotBeAbsentFromBundle
        )
    ));

    let stray_gap = FoundationalDiagnosticNamedGap::new(
        worth_foundational::FoundationalDiagnosticGapClass::CoverageOmission,
        worth_foundational::FoundationalDiagnosticGapTarget::Subject(
            worth_foundational::foundational_diagnostic_branch_discard_subject(
                worth_foundational::FoundationalBranchId::new("feature/stray").expect("branch"),
            ),
        ),
        worth_foundational::FoundationalDiagnosticGapClosurePosture::DebtNamed,
    );

    assert!(matches!(
        certify_current_basis_diagnostic_bundle(
            version("m6.phase5.stray_gap"),
            current_basis_receipt_source(),
            partial_explanation_bundle(),
            partial_explanation_coverage_matrix(stray_gap),
            certification_authority(),
        ),
        TransitionOutcome::Denied(
            FoundationalDiagnosticCertifiedAttachmentDenial::TypedNamedGapMustBelongToBundle
        )
    ));
}
