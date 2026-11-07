use super::super::*;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::lower_runtime_routing::{
    worth_query_lower_runtime_support_matrix, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRoutePlan,
    WorthQueryLowerRuntimeRouteSubjectIdentity, WorthQueryLowerRuntimeSubjectIdentity,
};

#[test]
fn route_plan_envelope_preserves_inventory_authority_and_support() {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Signal,
        "frontier-evidence-intake",
        WorthQueryLowerRuntimeSubjectIdentity::compose("test-subject")
            .field_value(WorthQueryEvidenceTag::new("test_subject"), "subject-1")
            .seal()
            .expect("fixture digest must fit"),
    )
    .expect("fixture digest must fit");
    let detail_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(WorthQueryEvidenceTag::new("test_detail"), "detail-1")
            .seal()
            .expect("fixture digest must fit");
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request,
        &detail_identity,
    )
    .expect("fixture digest must fit");
    let plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility,
        WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "test-route",
            &detail_identity,
        )
        .expect("fixture digest must fit"),
    );
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "envelope-route-test",
            &WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(WorthQueryEvidenceTag::new("test_retained"), "evidence-1")
            .seal()
            .expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit");
    let boundary =
        WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(&plan, &retained_evidence);
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake,
        &plan,
        &boundary,
        &retained_evidence,
    );
    let support_matrix = worth_query_lower_runtime_support_matrix();
    let support = support_matrix
        .support_for(WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake)
        .expect("frontier support row must exist");

    assert_eq!(
        envelope.authority_owner(),
        WorthQueryLowerRuntimeAuthorityOwner::Signal
    );
    assert_eq!(envelope.support_posture(), support.posture());
    assert_eq!(
        envelope.route_cost_posture(),
        WorthQueryLowerRuntimeCostPosture::QueryBoundaryAdapter
    );
}

#[test]
fn readmission_envelope_keeps_handoff_failure_topology_distinct() {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission,
        WorthQueryLowerRuntimeRouteKind::ReadmissionHandoff,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        "live-view-schema-admission",
        WorthQueryLowerRuntimeSubjectIdentity::compose("test-subject")
            .field_value(WorthQueryEvidenceTag::new("test_subject"), "subject-2")
            .seal()
            .expect("fixture digest must fit"),
    )
    .expect("fixture digest must fit");
    let detail_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(WorthQueryEvidenceTag::new("test_detail"), "detail-2")
            .seal()
            .expect("fixture digest must fit");
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request,
        &detail_identity,
    )
    .expect("fixture digest must fit");
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "envelope-test",
            &WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(WorthQueryEvidenceTag::new("test_retained"), "evidence-2")
            .seal()
            .expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit");
    let readmission =
        WorthQueryLowerRuntimeReadmissionReceipt::new(eligibility, &retained_evidence);
    let boundary =
        WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_readmission_receipt(&readmission);
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_readmission_receipt(
        WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission,
        &readmission,
        &boundary,
    );

    assert_eq!(
        envelope.route_failure_topology(),
        WorthQueryLowerRuntimeFailureTopology::ReadmissionHandoffBoundary
    );
    assert_eq!(
        envelope.route_kind(),
        WorthQueryLowerRuntimeRouteKind::ReadmissionHandoff
    );
}
