use super::*;

#[test]
fn runtime_public_support_matrix_freezes_stable_deferred_and_unsupported_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.support-matrix")
        .expect("task runtime should open a named workspace");
    let matrix = workspace.public_support_matrix();
    let contract = workspace.public_api_contract();

    assert_eq!(
        matrix.backend_posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(
        matrix.stable_row_count(),
        contract.stable_family_count() + 6
    );
    assert_eq!(
        matrix.deferred_row_count(),
        contract.deferred_family_count()
    );
    assert_eq!(
        matrix.unsupported_row_count(),
        contract.unsupported_family_count()
    );
    assert_eq!(
        matrix.parallel_api_forbidden_row_count(),
        matrix.rows().len()
    );
    assert_eq!(
        matrix.fail_closed_row_count(),
        matrix.deferred_row_count() + matrix.unsupported_row_count() + 6
    );

    let certification = matrix
        .row("authoritative-mutation-evidence-certification")
        .expect("authority-evidence gate row must be explicit");
    assert_eq!(certification.facade_family(), None);
    assert_eq!(
        certification.status(),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );
    assert_eq!(
        certification.owner_milestone(),
        "Runtime Authoritative Mutation Evidence Gate"
    );
    assert!(certification.parallel_api_forbidden());
    assert!(!certification.admission_fail_closed());
    assert!(certification.support_contract_digest().is_some());
    assert_eq!(
        certification.teaching_posture(),
        WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly
    );
    assert!(!certification.ordinary_downstream_dx());
    assert_eq!(
        certification.extension_rule(),
        "must-extend-target-binding-naming-continuity-causality-provenance-contract"
    );

    let shared_read_pinning = matrix
        .row("shared-read-pinning-boundary-closure")
        .expect("shared-read pinning boundary closure row must be explicit");
    assert_eq!(
        shared_read_pinning.support_contract_digest(),
        Some(
            WorthQuerySharedReadPinningCertification::support_gate_required()
                .closure()
                .closure_digest()
        )
    );
    assert_ne!(
        WorthQuerySharedReadPinningCertification::support_gate_required()
            .closure()
            .posture()
            .as_str(),
        "closed"
    );

    let milestone_nine_seven_closure = matrix
        .row("milestone-9.7-derived-closure-posture")
        .expect("milestone 9.7 derived closure row must be explicit");
    let expected_closure =
        WorthQueryMilestoneNineSevenDerivedClosure::support_profile_publication_contract();
    assert_eq!(
        milestone_nine_seven_closure.support_contract_digest(),
        Some(expected_closure.closure_digest())
    );
    assert_eq!(
        expected_closure.status(),
        WorthQueryMilestoneClosureStatus::Partial
    );
    assert_eq!(
        milestone_nine_seven_closure.owner_milestone(),
        "Milestone 9.7 Phase 18"
    );
    assert!(milestone_nine_seven_closure.admission_fail_closed());
    assert_eq!(
        milestone_nine_seven_closure.extension_rule(),
        "must-derive-milestone-closure-from-phase-local-postures"
    );

    let temporal = matrix
        .row_for_family(WorthQueryRuntimeFacadeFamily::Temporal)
        .expect("temporal support row must be explicit");
    assert_eq!(
        temporal.status(),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );
    assert_eq!(
        temporal.teaching_posture(),
        WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly
    );
    assert!(!temporal.ordinary_downstream_dx());
    assert!(temporal.parallel_api_forbidden());
    assert!(temporal.admission_fail_closed());
    assert_eq!(temporal.owner_milestone(), "Milestone 9.4");

    let intent = matrix
        .row_for_family(WorthQueryRuntimeFacadeFamily::Intent)
        .expect("intent vocabulary row must stay visible");
    assert_eq!(
        intent.status(),
        WorthQueryRuntimeFamilySupportStatus::Unsupported
    );
    assert_eq!(
        intent.teaching_posture(),
        WorthQueryRuntimeFamilyTeachingPosture::VisibleVocabularyOnly
    );
    assert!(!intent.ordinary_downstream_dx());
    assert!(intent.parallel_api_forbidden());
    assert!(intent.admission_fail_closed());
    assert_eq!(
        intent.extension_rule(),
        "must-admit-through-runtime-support-profile-before-public-use"
    );

    let temporal_async_certification = matrix
        .row("temporal-async-certification")
        .expect("temporal async certification row must stay explicit");
    assert_eq!(
        temporal_async_certification.status(),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );
    assert_eq!(
        temporal_async_certification.teaching_posture(),
        WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly
    );
    assert!(!temporal_async_certification.ordinary_downstream_dx());
    assert!(!temporal_async_certification.admission_fail_closed());

    let temporal_async_remask = matrix
        .row("temporal-async-remask")
        .expect("temporal async remask row must stay explicit");
    assert_eq!(
        temporal_async_remask.status(),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );
    assert_eq!(
        temporal_async_remask.teaching_posture(),
        WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly
    );
    assert!(!temporal_async_remask.ordinary_downstream_dx());
    assert!(temporal_async_remask.admission_fail_closed());
    assert_eq!(
        temporal_async_remask.extension_rule(),
        "must-remask-before-runtime-delivery-state-and-inspection-projection"
    );
}
