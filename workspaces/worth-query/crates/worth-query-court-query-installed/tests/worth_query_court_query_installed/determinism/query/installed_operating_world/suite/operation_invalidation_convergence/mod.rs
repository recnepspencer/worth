pub(crate) use worth_query_testkit_query::facade::query::suite::operation_invalidation_convergence::*;

#[test]
fn cert_reexecution_and_live_maintenance_converge_without_replay_authority_promotion() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        lineage_invalidation_workspace(request_execution, "invalidation-replay-convergence")
            .unwrap();
    let live = match settle_native(&mut workspace)
        .into_lifecycle()
        .promote(request_execution, &mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("lineage projection did not promote"),
    };
    let candidate = settle_native(&mut workspace).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("lineage projection did not share: {}", stop.detail())
        }
    };
    let (subject, candidate) = shared.into_leases();
    consume_empty_epoch(&mut workspace, &subject, &candidate);

    let original = bind(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let original_delivery = subject.drain(&mut workspace).unwrap();
    let _original_peer = candidate.drain(&mut workspace).unwrap();
    let original_impact = original_delivery.impact().semantic_projection();
    let original_observation = observe_delivery(&subject, original_delivery, &workspace);

    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        intent(),
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    let replay_delivery = subject.drain(&mut workspace).unwrap();
    let _replay_peer = candidate.drain(&mut workspace).unwrap();
    let replay_observation = observe_delivery(&subject, replay_delivery, &workspace);

    assert_eq!(&original_impact, original_observation.semantic.impact());
    assert!(original_observation
        .semantic
        .semantically_converges_with(&replay_observation.semantic));
    assert_eq!(
        original_observation.semantic.canonical_bytes(),
        replay_observation.semantic.canonical_bytes()
    );
    assert_eq!(
        original_observation.admitted_bytes,
        replay_observation.admitted_bytes
    );
    assert_eq!(
        original_observation.foundational,
        replay_observation.foundational
    );
    assert_eq!(
        original_observation.compatibility,
        domain::WorthQueryInvalidationCompatibilityOutcome::SharedEquivalentContinuity
    );
    assert_eq!(
        original_observation.compatibility,
        replay_observation.compatibility
    );
    assert_eq!(
        original_observation.maintenance_ordinal + 1,
        replay_observation.maintenance_ordinal
    );
    assert_eq!(original_observation.counters, replay_observation.counters);
    assert_eq!(
        original_observation.epoch_counters,
        replay_observation.epoch_counters
    );
}
