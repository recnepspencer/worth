use super::*;

#[test]
fn graph_summary_is_scope_explicit_and_canonical() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left = create_entity(execution, &runtime, "left");
    let right = create_entity(execution, &runtime, "right");
    let _relation = create_relation(execution, &runtime, left, right, "rel");

    let summary = runtime
        .inspect_what_happened()
        .graph_summary(execution, &current_graph_request(None, None, true));
    let kind_summary = runtime.inspect_what_happened().kind_summary(
        execution,
        &KindInspectionRequest {
            scope: InspectionScope::Current,
            partition_scope: None,
            kind_id: crate::facade::identity::KindId(1),
            record_class: crate::facade::inspection::InspectionRecordClass::Entity,
        },
    );

    assert_eq!(summary.entity_count, 2);
    assert_eq!(summary.relation_count, 1);
    assert_eq!(kind_summary.count, 2);
}

#[test]
fn current_graph_surfaces_match_version_and_snapshot_scopes_for_same_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left_a = crate::tests::support::create_entity_in_partition(
        execution,
        &runtime,
        "left-a",
        crate::facade::identity::PartitionId(7),
    );
    let left_b = crate::tests::support::create_entity_in_partition(
        execution,
        &runtime,
        "left-b",
        crate::facade::identity::PartitionId(7),
    );
    let right = crate::tests::support::create_entity_in_partition(
        execution,
        &runtime,
        "right",
        crate::facade::identity::PartitionId(11),
    );
    let left_relation = crate::tests::support::create_relation_in_partition(
        execution,
        &runtime,
        left_a,
        left_b,
        "left-rel",
        crate::facade::identity::PartitionId(7),
    );
    let _cross_relation = create_relation(execution, &runtime, left_b, right, "cross-rel");
    let snapshot = runtime.visibility_authority().snapshot();
    let version_id = runtime.current_version_id();

    let current_graph = runtime.inspect_what_happened().graph_summary(
        execution,
        &current_graph_request(
            Some(vec![crate::facade::identity::PartitionId(7)]),
            Some(vec![crate::facade::identity::KindId(2)]),
            true,
        ),
    );
    let version_graph = runtime.inspect_what_happened().graph_summary(
        execution,
        &version_graph_request(
            version_id,
            Some(vec![crate::facade::identity::PartitionId(7)]),
            Some(vec![crate::facade::identity::KindId(2)]),
            true,
        ),
    );
    let snapshot_graph = runtime.inspect_what_happened().graph_summary(
        execution,
        &snapshot_graph_request(
            InspectionScope::Snapshot(snapshot.clone()),
            Some(vec![crate::facade::identity::PartitionId(7)]),
            Some(vec![crate::facade::identity::KindId(2)]),
            true,
        ),
    );
    let current_connectivity = runtime.inspect_what_happened().connectivity_summary(
        execution,
        &connectivity_request(
            InspectionScope::Current,
            Some(vec![crate::facade::identity::PartitionId(7)]),
            Some(vec![crate::facade::identity::KindId(2)]),
            true,
        ),
    );
    let version_connectivity = runtime.inspect_what_happened().connectivity_summary(
        execution,
        &connectivity_request(
            InspectionScope::Version(version_id),
            Some(vec![crate::facade::identity::PartitionId(7)]),
            Some(vec![crate::facade::identity::KindId(2)]),
            true,
        ),
    );
    let snapshot_connectivity = runtime.inspect_what_happened().connectivity_summary(
        execution,
        &connectivity_request(
            InspectionScope::Snapshot(snapshot),
            Some(vec![crate::facade::identity::PartitionId(7)]),
            Some(vec![crate::facade::identity::KindId(2)]),
            true,
        ),
    );
    let neighbors_current =
        runtime
            .inspect_what_happened()
            .neighbors(execution, InspectionScope::Current, left_a);
    let neighbors_version = runtime.inspect_what_happened().neighbors(
        execution,
        InspectionScope::Version(version_id),
        left_a,
    );

    assert_eq!(current_graph.entity_count, version_graph.entity_count);
    assert_eq!(current_graph.entity_count, snapshot_graph.entity_count);
    assert_eq!(current_graph.relation_count, version_graph.relation_count);
    assert_eq!(current_graph.relation_count, snapshot_graph.relation_count);
    assert_eq!(current_graph.entity_kinds, version_graph.entity_kinds);
    assert_eq!(current_graph.entity_kinds, snapshot_graph.entity_kinds);
    assert_eq!(current_graph.relation_kinds, version_graph.relation_kinds);
    assert_eq!(current_graph.relation_kinds, snapshot_graph.relation_kinds);
    assert_eq!(
        current_connectivity.component_count,
        version_connectivity.component_count
    );
    assert_eq!(
        current_connectivity.component_count,
        snapshot_connectivity.component_count
    );
    assert_eq!(
        current_connectivity.largest_component_size,
        version_connectivity.largest_component_size
    );
    assert_eq!(
        current_connectivity.largest_component_size,
        snapshot_connectivity.largest_component_size
    );
    assert_eq!(
        current_connectivity.components,
        version_connectivity.components
    );
    assert_eq!(
        current_connectivity.components,
        snapshot_connectivity.components
    );
    assert_eq!(neighbors_current.outgoing_relation_ids, vec![left_relation]);
    assert_eq!(
        neighbors_current.outgoing_relation_ids,
        neighbors_version.outgoing_relation_ids
    );
}

#[test]
fn snapshot_graph_summary_fails_closed_when_snapshot_handle_is_unavailable() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "alpha");
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());

    let summary = runtime.inspect_what_happened().graph_summary(
        execution,
        &snapshot_graph_request(
            InspectionScope::Snapshot(created.snapshot.clone()),
            None,
            None,
            true,
        ),
    );

    assert_eq!(
        summary.scope,
        InspectionScope::Snapshot(created.snapshot.clone())
    );
    assert_eq!(summary.version_id, created.version_id);
    assert_eq!(summary.entity_count, 0);
    assert_eq!(summary.relation_count, 0);
    assert_eq!(
        summary.availability,
        InspectionAvailability::UnavailableByRetention
    );
}

#[test]
fn connectivity_summary_refuses_oversized_budget_with_explicit_degradation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left = create_entity(execution, &runtime, "left");
    let right = create_entity(execution, &runtime, "right");
    let _relation = create_relation(execution, &runtime, left, right, "rel");

    let summary = runtime.inspect_what_happened().connectivity_summary(
        execution,
        &crate::facade::inspection::ConnectivityInspectionRequest {
            scope: InspectionScope::Current,
            partition_scope: None,
            relation_kind_scope: None,
            include_members: false,
            budget: crate::facade::inspection::ConnectivityInspectionBudget {
                max_entities: 1,
                max_relations: 1,
                max_frontier: 1,
                max_components: 1,
                max_work_units: 1,
            },
        },
    );

    assert_eq!(
        summary.availability,
        InspectionAvailability::UnavailableByBudget
    );
    assert!(summary
        .degradations
        .contains(&crate::facade::inspection::InspectionDegradation::WorkBudgetExceeded));
}

#[test]
fn retention_summary_refuses_work_budget_with_explicit_degradation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "retained");
    let _relation = create_relation(execution, &runtime, entity, entity, "loop");

    let summary = runtime.inspect_what_happened().retention_summary(
        &crate::facade::inspection::RetentionInspectionRequest {
            max_entity_slots_scanned: 32,
            max_relation_slots_scanned: 32,
            max_work_units: 1,
        },
    );

    assert_eq!(
        summary.availability,
        InspectionAvailability::UnavailableByBudget
    );
    assert!(summary
        .degradations
        .contains(&crate::facade::inspection::InspectionDegradation::WorkBudgetExceeded));
}

#[test]
fn inspection_resolution_context_keeps_no_context_explicitly() {
    assert_eq!(
        InspectionResolutionContext::NoContext,
        InspectionResolutionContext::NoContext
    );
}

#[test]
fn inspection_counts_preserve_wide_u64_values() {
    let summary = crate::facade::inspection::GraphInspectionSummary {
        scope: InspectionScope::Current,
        version_id: crate::facade::identity::VersionId(7),
        partition_count: u64::MAX,
        entity_count: u64::MAX,
        relation_count: u64::MAX,
        entity_kinds: Vec::new(),
        relation_kinds: Vec::new(),
        origin: InspectionOrigin::CurrentTruth,
        access_path: InspectionAccessPath::DirectLookup,
        availability: InspectionAvailability::Direct,
        degradations: Vec::new(),
    };
    assert_eq!(summary.partition_count, u64::MAX);
    assert_eq!(summary.entity_count, u64::MAX);
    assert_eq!(summary.relation_count, u64::MAX);
}
