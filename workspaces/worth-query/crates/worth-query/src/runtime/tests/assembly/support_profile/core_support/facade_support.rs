use super::*;

#[test]
fn runtime_support_profiles_expose_facade_family_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let primary_runtime = stateful_bridge_task_runtime(request_execution);
    let bridge_runtime = complete_backend_from_parts_builder()
        .build_backend_from_parts()
        .build(request_execution)
        .expect("complete backend parts should build");

    for family in [
        WorthQueryRuntimeFacadeFamily::Read,
        WorthQueryRuntimeFacadeFamily::Live,
        WorthQueryRuntimeFacadeFamily::Computed,
        WorthQueryRuntimeFacadeFamily::SharedRead,
        WorthQueryRuntimeFacadeFamily::Submission,
        WorthQueryRuntimeFacadeFamily::Effect,
        WorthQueryRuntimeFacadeFamily::BranchPreview,
        WorthQueryRuntimeFacadeFamily::Write,
        WorthQueryRuntimeFacadeFamily::Inspect,
    ] {
        assert_eq!(
            primary_runtime
                .support_profile()
                .support_for(family)
                .expect("primary support row should exist")
                .status(),
            WorthQueryRuntimeFamilySupportStatus::Supported
        );
        assert_eq!(
            bridge_runtime
                .support_profile()
                .support_for(family)
                .expect("bridge-backed support row should exist")
                .status(),
            WorthQueryRuntimeFamilySupportStatus::Supported
        );
    }

    assert_eq!(
        primary_runtime.support_profile().posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(
        bridge_runtime.support_profile().posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );

    let bridge_support_profile = bridge_runtime.support_profile();
    let write_support = bridge_support_profile
        .support_for(WorthQueryRuntimeFacadeFamily::Write)
        .expect("write support row should exist");
    assert!(write_support
        .evidence()
        .iter()
        .any(|evidence| evidence == "authoritative-mutation-evidence"));

    let inspect_support = bridge_support_profile
        .support_for(WorthQueryRuntimeFacadeFamily::Inspect)
        .expect("inspect support row should exist");
    assert!(inspect_support
        .authority_lanes()
        .contains(&WorthQueryAuthorityLane::BranchLocalTruth));
    assert!(inspect_support
        .authority_lanes()
        .contains(&WorthQueryAuthorityLane::PendingWriteIntent));
}

#[test]
fn runtime_public_api_contract_closes_runtime_backed_temporal_async_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_runtime(request_execution);
    let contract = runtime.public_api_contract();

    assert_eq!(
        contract.backend_posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(contract.deferred_family_count(), 2);
    assert!(!contract.contract_digest().is_empty());

    for family in [
        WorthQueryRuntimeFacadeFamily::Temporal,
        WorthQueryRuntimeFacadeFamily::AsyncResource,
        WorthQueryRuntimeFacadeFamily::MixedCauseDelivery,
    ] {
        let row = contract
            .family(family)
            .expect("runtime-backed support row should exist");
        assert_eq!(
            row.status(),
            WorthQueryRuntimeFamilySupportStatus::Supported
        );
        assert_eq!(
            row.teaching_posture(),
            WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly
        );
        assert!(!row.ordinary_downstream_dx());
        assert!(row.parallel_api_forbidden());
        assert!(row.admission_fail_closed());
        assert_eq!(row.owner_closure(), "Milestone 9.4");
        assert_eq!(
            row.extension_rule(),
            "must-extend-stabilized-handle-state-lane-aspect-inspection-facade"
        );
        assert!(row.reason().is_none());
        assert_eq!(row.authority_lanes().len(), 1);
        assert_eq!(row.evidence().len(), 1);
    }

    for (family, expected_reason) in [
        (
            WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
            "Milestone 10",
        ),
        (
            WorthQueryRuntimeFacadeFamily::DurableArtifacts,
            "Milestone 11",
        ),
    ] {
        let row = contract
            .family(family)
            .expect("deferred support gate row should exist");
        assert_eq!(
            row.status(),
            WorthQueryRuntimeFamilySupportStatus::DeferredDebt
        );
        assert_eq!(
            row.teaching_posture(),
            WorthQueryRuntimeFamilyTeachingPosture::VisibleButDeferred
        );
        assert!(!row.ordinary_downstream_dx());
        assert!(row.parallel_api_forbidden());
        assert!(row.admission_fail_closed());
        assert_eq!(row.owner_closure(), expected_reason);
        assert!(row
            .reason()
            .is_some_and(|reason| reason.contains(expected_reason)));
        assert!(row.authority_lanes().is_empty());
        assert!(row.evidence().is_empty());
    }

    let read = contract
        .family(WorthQueryRuntimeFacadeFamily::Read)
        .expect("supported read family should exist");
    assert_eq!(
        read.teaching_posture(),
        WorthQueryRuntimeFamilyTeachingPosture::OrdinaryRuntimeDx
    );
    assert!(read.ordinary_downstream_dx());
    assert!(!read.admission_fail_closed());

    let intent = contract
        .family(WorthQueryRuntimeFacadeFamily::Intent)
        .expect("intent family should remain visible in the public contract");
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
}
