use super::*;
use crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity;
use crate::runtime::WorthQueryWriteCommand;

use super::write_authority_receipt::WRITE_AUTHORITY_CAPABILITY_LABEL;

#[test]
fn route_plan_drift_rejects_foreign_boundary_receipt() {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        WRITE_AUTHORITY_CAPABILITY_LABEL,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "test-subject",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("test_subject"),
            "subject-a",
        )
        .seal()
        .expect("fixture digest must fit"),
    )
    .expect("fixture digest must fit");
    let detail_a = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("test_detail"),
        "detail-a",
    )
    .seal()
    .expect("fixture digest must fit");
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request, &detail_a,
    )
    .expect("fixture digest must fit");
    let plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility,
        WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity("test-route", &detail_a)
            .expect("fixture digest must fit"),
    );

    let foreign_request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        SIGNAL_INVALIDATION_CAPABILITY_LABEL,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "test-subject",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("test_subject"),
            "subject-b",
        )
        .seal()
        .expect("fixture digest must fit"),
    )
    .expect("fixture digest must fit");
    let detail_b = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("test_detail"),
        "detail-b",
    )
    .seal()
    .expect("fixture digest must fit");
    let foreign_eligibility =
        WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
            foreign_request,
            &detail_b,
        )
        .expect("fixture digest must fit");
    let foreign_plan = WorthQueryLowerRuntimeRoutePlan::new(
        foreign_eligibility,
        WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity("test-route", &detail_b)
            .expect("fixture digest must fit"),
    );
    let retained_a =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "runtime-backend-test",
            &crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(
                crate::evidence_identity::WorthQueryEvidenceTag::new("test_retained"),
                "detail-a",
            )
            .seal()
            .expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit");
    let retained_b =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "runtime-backend-test",
            &crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(
                crate::evidence_identity::WorthQueryEvidenceTag::new("test_retained"),
                "detail-b",
            )
            .seal()
            .expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit");
    let foreign_receipt =
        WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(&foreign_plan, &retained_b);

    let drift = foreign_receipt
        .drift_from_route_plan(&plan, &retained_a)
        .expect("foreign route receipt must drift");

    assert!(drift.contains("boundary execution request digest"));
}

#[test]
fn write_authority_boundary_receipt_carries_boundary_envelope() {
    let command = WorthQueryWriteCommand::Delete {
        entity_identity: crate::memory_workspace::admit_authored_entity_label("task-1")
            .expect("fixture digest must fit"),
    };
    let mutation_receipt = WorthQueryMutationReceipt::from_authoritative_parts(
        crate::memory_workspace::admit_external_commit_label("commit-1")
            .expect("fixture digest must fit"),
        crate::memory_workspace::admit_external_snapshot_label("snapshot-1")
            .expect("fixture digest must fit"),
        Vec::new(),
    );
    let receipt = WriteAuthorityExecutionReceipt::from_command(&command, mutation_receipt);

    assert_eq!(
        receipt.boundary_envelope().seam_key(),
        WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution
    );
    assert_eq!(
        receipt.boundary_envelope().boundary_execution_identity(),
        receipt
            .boundary_execution_receipt()
            .boundary_execution_identity()
    );
}
