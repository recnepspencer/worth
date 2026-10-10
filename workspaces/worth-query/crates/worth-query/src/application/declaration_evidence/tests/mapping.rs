use worth_foundational::facade::{
    FoundationalBoundaryEvidenceCloseoutDisposition, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceReceiptKind, FoundationalBoundaryEvidenceSupportAttachment,
    FoundationalBoundaryEvidenceSupportBasisDisclosure,
    FoundationalBoundaryEvidenceSupportResidualDebtKind,
    FoundationalBoundaryEvidenceSupportTruthKind,
};

use crate::application::{
    WorthQueryDeclarationFoundationalEvidenceClass, WorthQueryDeclarationFoundationalEvidenceInput,
};

use super::domain::{
    admitted_handle, Declaration, DeferredFamily, FailedFamily, IllegalRoleFamily, LegalFamily,
    RebindRequiredFamily, StaleFamily,
};

#[test]
fn legality_evidence_maps_to_planning_receipt_and_published_support() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let evidence = handle
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::legality_evidence(
                handle
                    .declare_and_review(Declaration::<LegalFamily>::new("edge:42"))
                    .unwrap_or_else(|_| panic!("legality should pass")),
            ),
        )
        .expect("foundational description should admit");

    assert_eq!(
        evidence.class(),
        WorthQueryDeclarationFoundationalEvidenceClass::LegalityAdmitted
    );
    assert_eq!(
        evidence.provenance().freshness_posture(),
        FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
    );
    assert_eq!(
        evidence
            .planning_receipt()
            .expect("planning receipt should be retained")
            .receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Planning
    );
    assert!(evidence.receipt().is_none());
    match evidence
        .support_attachment()
        .expect("support attachment should exist")
    {
        FoundationalBoundaryEvidenceSupportAttachment::Published(artifact) => {
            assert_eq!(
                artifact.support_truth_kind(),
                FoundationalBoundaryEvidenceSupportTruthKind::EvidenceBundle
            );
            assert_eq!(
                artifact.basis_disclosure(),
                FoundationalBoundaryEvidenceSupportBasisDisclosure::CompleteBasis
            );
        }
        _ => panic!("expected published support artifact"),
    }
}

#[test]
fn denied_and_degraded_paths_stay_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let declaration = handle
        .declare(Declaration::<IllegalRoleFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("declaration should admit"));
    let legality_denied = handle
        .review_legality(declaration)
        .err()
        .unwrap_or_else(|| panic!("legality should deny"));
    let evidence = handle
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::legality_denial(legality_denied),
        )
        .expect("denied evidence should still describe");

    assert_eq!(
        evidence.class(),
        WorthQueryDeclarationFoundationalEvidenceClass::LegalityDenied
    );
    assert_eq!(
        evidence
            .receipt()
            .expect("denied path should retain completed receipt")
            .closeout_disposition(),
        Some(FoundationalBoundaryEvidenceCloseoutDisposition::Denied)
    );
    match evidence
        .support_attachment()
        .expect("support attachment should exist")
    {
        FoundationalBoundaryEvidenceSupportAttachment::Closeout(artifact) => {
            assert_eq!(
                artifact.support_truth_kind(),
                FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
            );
        }
        _ => panic!("expected closeout support artifact"),
    }
}

#[test]
fn deferred_stale_rebind_and_failed_progression_map_honestly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let collaborative = admitted_handle(request_execution, "collaborative");
    let restricted = admitted_handle(request_execution, "restricted");

    let deferred = collaborative
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::progression_checked(
                collaborative.progress_declaration_checked(
                    collaborative
                        .declare_and_review(Declaration::<DeferredFamily>::new("edge:42"))
                        .unwrap_or_else(|_| panic!("legality should pass")),
                ),
            ),
        )
        .expect("deferred progression should still describe");
    let stale = collaborative
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::progression_checked(
                collaborative.progress_declaration_checked(
                    collaborative
                        .declare_and_review(Declaration::<StaleFamily>::new("edge:42"))
                        .unwrap_or_else(|_| panic!("legality should pass")),
                ),
            ),
        )
        .expect("stale progression should still describe");
    let rebind = restricted
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::progression_checked(
                restricted.progress_declaration_checked(
                    restricted
                        .declare_and_review(Declaration::<RebindRequiredFamily>::new("edge:42"))
                        .unwrap_or_else(|_| panic!("legality should pass")),
                ),
            ),
        )
        .expect("rebind progression should still describe");
    let failed = collaborative
        .describe_foundational(
            WorthQueryDeclarationFoundationalEvidenceInput::progression_checked(
                collaborative.progress_declaration_checked(
                    collaborative
                        .declare_and_review(Declaration::<FailedFamily>::new("edge:42"))
                        .unwrap_or_else(|_| panic!("legality should pass")),
                ),
            ),
        )
        .expect("failed progression should still describe");

    match deferred
        .support_attachment()
        .expect("support attachment should exist")
    {
        FoundationalBoundaryEvidenceSupportAttachment::Published(artifact) => {
            assert_eq!(
                artifact.support_truth_kind(),
                FoundationalBoundaryEvidenceSupportTruthKind::ResidualDebtStatement
            );
            assert_eq!(
                artifact.residual_debt().expect("debt should exist").kinds(),
                &[FoundationalBoundaryEvidenceSupportResidualDebtKind::RebuildRequired]
            );
        }
        _ => panic!("expected published deferred support"),
    }
    assert_eq!(
        stale.provenance().freshness_posture(),
        FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained
    );
    assert_eq!(
        rebind
            .receipt()
            .expect("rebind should retain blocked receipt")
            .closeout_disposition(),
        Some(FoundationalBoundaryEvidenceCloseoutDisposition::Blocked)
    );
    match failed
        .support_attachment()
        .expect("support attachment should exist")
    {
        FoundationalBoundaryEvidenceSupportAttachment::Closeout(artifact) => {
            assert_eq!(
                artifact.support_truth_kind(),
                FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
            );
        }
        _ => panic!("expected closeout failed support"),
    }
}
