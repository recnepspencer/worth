use super::*;

#[test]
fn read_execution_eligibility_admits_runtime_current_route() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("read-intent-eligibility", runtime)
        .expect("workspace should build");
    let family = identity_read_family(&mut workspace, "tasks");
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::read_family_entrypoint(
            crate::intent_admission::WorthQueryReadExecutionIntentSeed::current_runtime(family),
        )
        .expect("read request should build");
    let eligibility =
        crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(request);

    assert_eq!(
        eligibility.support_posture(),
        WorthQueryIntentAdmissionSupportEligibility::Admitted
    );
    assert_eq!(
        eligibility.capability_posture(),
        WorthQueryIntentAdmissionCapabilityEligibility::Admitted
    );
    assert_eq!(
        eligibility.basis_posture(),
        WorthQueryIntentAdmissionBasisEligibility::ReadExecutionCurrentRuntimeAdmitted
    );
    assert_eq!(
        eligibility.routing_support_posture(),
        WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
            crate::facade::runtime::WorthQueryIntentAdmissionExecutionSeam::QueryRuntimeReadExecutionRoute
        )
    );
    assert_eq!(
        eligibility.pre_decision_posture(),
        WorthQueryIntentAdmissionPreDecisionPosture::Admitted
    );
}

#[test]
fn read_execution_basis_context_mismatch_stops_as_phase_two_violation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("read-intent-basis-mismatch", runtime)
        .expect("workspace should build");
    let family = identity_read_family(&mut workspace, "tasks");
    let mut unrelated_workspace =
        WorthQueryWorkspace::new("read-intent-unrelated", read_runtime(request_execution))
            .expect("workspace should build");
    let unrelated_family = profile_read_family(&mut unrelated_workspace, "other-tasks");
    let mismatched_context = current_context_for_family(&unrelated_family, "snapshot:mismatch");
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::read_family_in_basis_context_entrypoint(
            crate::intent_admission::WorthQueryReadExecutionIntentSeed::in_basis_context(
                family,
                mismatched_context,
            ),
        )
        .expect("basis-context request should build");
    let eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    );
    let decision = admit_runtime_intent_request(request);

    assert_eq!(
        eligibility.capability_posture(),
        WorthQueryIntentAdmissionCapabilityEligibility::Violation {
            stage: "read-basis-context-admission",
            detail: "read-basis-context query digest does not match requested read family",
        }
    );
    assert_eq!(
        eligibility.basis_posture(),
        WorthQueryIntentAdmissionBasisEligibility::ReadExecutionBasisContextViolation(
            "read-basis-context query digest does not match requested read family"
        )
    );
    match decision {
        WorthQueryIntentAdmissionDecision::Violation(violation) => {
            assert_eq!(violation.stage(), "read-basis-context-admission");
            assert_eq!(
                violation.message(),
                "read-basis-context query digest does not match requested read family"
            );
        }
        other => panic!("expected read basis mismatch violation, got {other:?}"),
    }
}
