pub(crate) use worth_query_testkit_query::facade::query::suite::operation_invalidation_routing::*;

#[test]
fn same_aspect_other_field_ordinary_consumers_do_not_enter_installed_owner_routing() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Invalidation,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace(request_execution, "invalidation-same-aspect-field-routing")
        .unwrap();
    let inserted = workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "seed")
        })
        .unwrap();
    let entity = inserted.deltas()[0].entity_identity().clone();
    let (subject, candidate) = shared_pair(request_execution, &mut workspace);
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    let ordinary = (0..64)
        .map(|ordinal| ordinary_name_view(request_execution, &mut workspace, ordinal))
        .collect::<Vec<_>>();

    workspace
        .update(request_execution, entity, |mutation| {
            mutation.aspect("identity.id", "affected-installed-owner")
        })
        .unwrap();

    for view in ordinary {
        assert!(workspace.observe(&view).query_delivery_batches.is_empty());
    }
    let subject_delivery = subject.drain(&mut workspace).unwrap();
    let candidate_delivery = candidate.drain(&mut workspace).unwrap();
    let delta = subject
        .consumer_invalidation_delta(subject_delivery)
        .unwrap();
    assert!(candidate
        .consumer_invalidation_delta(candidate_delivery)
        .is_ok());
    assert_eq!(delta.epoch_counters().capability_index_lookups, 1);
    assert_eq!(delta.epoch_counters().live_target_candidates_visited, 1);
    assert_eq!(delta.epoch_counters().installed_route_index_probes, 1);
    assert_eq!(delta.epoch_counters().installed_candidates_skipped, 0);
}

#[test]
fn installed_ordering_dependency_routes_end_to_end_when_the_live_read_omits_it() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = impact_collection_invalidation_workspace(
        request_execution,
        "invalidation-installed-only-ordering-routing",
    )
    .unwrap();
    let inserted = workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation
                .aspect("identity.id", "installed-ordering-row")
                .aspect("ordering.position", "first")
        })
        .unwrap();
    let entity = inserted.deltas()[0].entity_identity().clone();
    let live = match settle_installed_ordering(&mut workspace)
        .into_lifecycle()
        .promote(request_execution, &mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("installed ordering subject did not promote"),
    };
    let candidate = settle_installed_ordering(&mut workspace).into_lifecycle();
    let (subject, candidate) = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared.into_leases(),
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("installed ordering sharing stopped: {}", stop.detail())
        }
    };
    assert!(subject.drain(&mut workspace).unwrap().delivery().is_empty());
    assert!(candidate
        .drain(&mut workspace)
        .unwrap()
        .delivery()
        .is_empty());

    workspace
        .update(request_execution, entity, |mutation| {
            mutation.aspect("ordering.position", "second")
        })
        .unwrap();

    let delivery = subject.drain(&mut workspace).unwrap();
    let peer = candidate.drain(&mut workspace).unwrap();
    let delta = subject.consumer_invalidation_delta(delivery).unwrap();
    let peer_delta = candidate.consumer_invalidation_delta(peer).unwrap();
    assert_eq!(
        delta.impact().class(),
        domain::WorthQueryImpactClass::WindowShift
    );
    assert_eq!(
        delta.impact().affected_roles(),
        [
            domain::WorthQuerySemanticDependencyRole::Ordering,
            domain::WorthQuerySemanticDependencyRole::WindowBoundary,
        ]
    );
    assert!(delta.shares_epoch_with(&peer_delta));
    assert_eq!(
        delta.epoch_counters().installed_target_candidates_selected,
        1
    );
    assert_eq!(delta.epoch_counters().live_target_candidates_visited, 1);
    assert_eq!(delta.epoch_counters().capability_index_lookups, 1);
}
