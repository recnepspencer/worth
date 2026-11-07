use super::*;

#[test]
fn basis_observation_eligibility_admits_without_runtime_handoff() {
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::basis_observation_lane(
            crate::basis_lifecycle::RawBasisIntent::CurrentHead,
        )
        .expect("basis observation request should build");
    let eligibility =
        crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(request)
            .expect("fixture digest must fit");

    assert_eq!(
        eligibility.support_posture(),
        WorthQueryIntentAdmissionSupportEligibility::Admitted
    );
    assert_eq!(
        eligibility.basis_posture(),
        WorthQueryIntentAdmissionBasisEligibility::ObservationLifecycleAdmitted
    );
    assert_eq!(
        eligibility.routing_support_posture(),
        WorthQueryIntentAdmissionRoutingSupportEligibility::NoExecutionHandoff(
            "basis-observation-admitted-plan-scopes-to-lower-runtime-evidence-without-query-execution-handoff"
        )
    );
    assert_eq!(eligibility.admitted_execution_seam(), None);
}

#[test]
fn projection_consumption_warning_eligibility_stays_admitted_without_execution_handoff() {
    let declaration = crate::projection_consumption::declare_projection_consumption(
        crate::projection_consumption::ProjectionConsumptionSource::test_only(
            crate::projection_consumption::ProjectionSourceFamily::QueryContextExecution,
            Some("query-digest"),
            Some("basis-digest"),
            Some("result-digest"),
            Some("shape-digest"),
            "query-context:test",
        ),
        crate::projection_consumption::ProjectionConsumptionBindingContext::test_only_with_projection_metadata(
            "shape-digest",
            "query-digest",
            "shape-digest",
            "projection.identity",
            "narrowed-shape-digest",
            "policy-digest",
            "tenant-schema-digest",
            crate::projection_consumption::test_authorized_field_paths(&["field.visible"]),
        ),
        crate::projection_consumption::ProjectMaterializedFacts::declare()
            .display_field_path(crate::projection_consumption::projection_fact_field_path_from_segments([worth_foundational::facade::FieldKey::new("field").expect("projection fact field segment should admit"), worth_foundational::facade::FieldKey::new("visible").expect("projection fact field segment should admit")])),
    )
    .expect("projection declaration should build");
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::projection_consumption(
            declaration,
        )
        .expect("projection request should build");
    let eligibility =
        crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(request)
            .expect("fixture digest must fit");

    assert_eq!(
        eligibility.projection_source_posture(),
        WorthQueryIntentAdmissionProjectionSourceEligibility::ProjectionConsumptionAdmittedWithWarnings(
            "projection-consumption-warning-bearing-admission"
        )
    );
    assert_eq!(eligibility.admitted_execution_seam(), None);
}
