use crate::facade::inspection::{
    ConnectivityInspectionRequest, GraphInspectionRequest, InspectionRecordClass, InspectionScope,
    KindInspectionRequest, RecentCommitInspectionRequest, RetentionInspectionRequest,
    StructuralIdentityQueryRequest,
};
use crate::identity::data::KindId;
use crate::symbols::data::Symbol;
use crate::tests::support::*;

#[test]
fn complexity_budget_graph_summary_reports_explicit_inspection_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left = create_entity(execution, &runtime, "left");
    let right = create_entity(execution, &runtime, "right");
    let _relation = create_relation(execution, &runtime, left, right, "rel");

    runtime.performance_access().reset_counters();
    let summary = runtime
        .inspect_what_happened()
        .graph_summary(
            execution,
            &GraphInspectionRequest {
                scope: InspectionScope::Current,
                partition_scope: None,
                relation_kind_scope: None,
                summary_only: true,
                budget: crate::facade::inspection::GraphInspectionBudget {
                    max_entities: 32,
                    max_relations: 32,
                    max_work_units: 128,
                },
            },
        )
        .expect("read request remains live");
    let counters = runtime.performance_access().counters();

    assert_eq!(summary.entity_count, 2);
    assert_eq!(summary.relation_count, 1);
    assert_eq!(counters.inspection_graph_summary_requests, 1);
    assert_eq!(
        summary.origin,
        crate::facade::inspection::InspectionOrigin::CurrentTruth
    );
    assert_eq!(
        summary.access_path,
        crate::facade::inspection::InspectionAccessPath::DirectLookup
    );
    assert_eq!(
        counters.visible_authoritative_entity_records_materialized,
        0
    );
    assert_eq!(
        counters.visible_authoritative_relation_records_materialized,
        0
    );
    assert_eq!(counters.visibility_entity_slot_scans, 0);
    assert_eq!(counters.visibility_relation_slot_scans, 0);
}

#[test]
fn complexity_budget_structural_identity_distinguishes_direct_lookup_from_broad_query() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "alpha");
    let _other = create_entity(execution, &runtime, "beta");

    runtime.performance_access().reset_counters();
    let direct = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(entity),
        )
        .expect("read request remains live");
    let direct_counters = runtime.performance_access().counters();

    assert!(direct.is_some());
    assert_eq!(direct_counters.inspection_structural_identity_lookups, 1);
    assert_eq!(
        direct_counters.inspection_structural_identity_query_scans,
        0
    );
    assert_eq!(direct_counters.visibility_entity_slot_scans, 0);

    runtime.performance_access().reset_counters();
    let broad = runtime
        .inspect_what_happened()
        .query_structural_identity(
            execution,
            &StructuralIdentityQueryRequest {
                scope: InspectionScope::Current,
                partition_scope: None,
                fingerprint_family: Symbol(1),
            },
        )
        .expect("read request remains live");
    let broad_counters = runtime.performance_access().counters();

    assert!(broad.is_empty());
    assert_eq!(broad_counters.inspection_structural_identity_query_scans, 1);
    assert!(broad_counters.inspection_structural_identity_lookups >= 2);
    assert!(broad_counters.visible_authoritative_entity_records_materialized >= 2);
}

#[test]
fn complexity_budget_kind_summary_reports_request_shaped_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _left_a = create_entity_in_partition(execution, &runtime, "left-a", PartitionId(7));
    let _left_b = create_entity_in_partition(execution, &runtime, "left-b", PartitionId(7));
    let _right = create_entity_in_partition(execution, &runtime, "right", PartitionId(11));

    runtime.performance_access().reset_counters();
    let summary = runtime
        .inspect_what_happened()
        .kind_summary(
            execution,
            &KindInspectionRequest {
                scope: InspectionScope::Current,
                partition_scope: Some(vec![PartitionId(7)]),
                kind_id: KindId(1),
                record_class: InspectionRecordClass::Entity,
            },
        )
        .expect("read request remains live");
    let counters = runtime.performance_access().counters();

    assert_eq!(summary.count, 2);
    assert_eq!(summary.touched_partitions, vec![PartitionId(7)]);
    assert_eq!(counters.inspection_kind_summary_requests, 1);
    assert_eq!(counters.visibility_entity_slot_scans, 0);
    assert_eq!(
        counters.visible_authoritative_relation_records_materialized,
        0
    );
}

#[test]
fn complexity_budget_connectivity_summary_reports_broad_traversal_work_explicitly() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left = create_entity(execution, &runtime, "left");
    let right = create_entity(execution, &runtime, "right");
    let isolated = create_entity(execution, &runtime, "isolated");
    let _relation = create_relation(execution, &runtime, left, right, "rel");

    runtime.performance_access().reset_counters();
    let summary = runtime
        .inspect_what_happened()
        .connectivity_summary(
            execution,
            &ConnectivityInspectionRequest {
                scope: InspectionScope::Current,
                partition_scope: None,
                relation_kind_scope: None,
                include_members: false,
                budget: crate::facade::inspection::ConnectivityInspectionBudget {
                    max_entities: 32,
                    max_relations: 32,
                    max_frontier: 32,
                    max_components: 32,
                    max_work_units: 256,
                },
            },
        )
        .expect("read request remains live");
    let counters = runtime.performance_access().counters();

    assert_eq!(summary.component_count, 2);
    assert_eq!(summary.largest_component_size, 2);
    assert_eq!(summary.enumerated_entity_count, 3);
    assert_eq!(
        summary.origin,
        crate::facade::inspection::InspectionOrigin::CurrentTruth
    );
    assert_eq!(
        summary.access_path,
        crate::facade::inspection::InspectionAccessPath::DirectLookup
    );
    assert_eq!(
        counters.visible_authoritative_entity_records_materialized,
        0
    );
    assert_eq!(
        counters.visible_authoritative_relation_records_materialized,
        0
    );
    assert_eq!(counters.visibility_entity_slot_scans, 0);
    assert_eq!(counters.visibility_relation_slot_scans, 0);
    assert!(summary
        .components
        .iter()
        .all(|component| component.members.is_none()));
    assert_eq!(isolated.partition_id, PartitionId::main());
}

#[test]
fn complexity_budget_retention_summary_reports_bounded_slot_scans() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left = create_entity(execution, &runtime, "left");
    let right = create_entity(execution, &runtime, "right");
    let _relation = create_relation(execution, &runtime, left, right, "rel");

    runtime.performance_access().reset_counters();
    let summary = runtime
        .inspect_what_happened()
        .retention_summary(&RetentionInspectionRequest {
            max_entity_slots_scanned: 32,
            max_relation_slots_scanned: 32,
            max_work_units: 128,
        });
    let counters = runtime.performance_access().counters();

    assert_eq!(
        summary.availability,
        crate::facade::inspection::InspectionAvailability::Direct
    );
    assert!(counters.inspection_retention_entity_slot_scans >= 2);
    assert!(counters.inspection_retention_relation_slot_scans >= 1);
    assert_eq!(counters.inspection_budget_refusals, 0);
}

#[test]
fn complexity_budget_neighbor_inspection_uses_adjacency_not_relation_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation = create_relation(execution, &runtime, source, target, "rel");

    runtime.performance_access().reset_counters();
    let neighbors = runtime
        .inspect_what_happened()
        .neighbors(execution, InspectionScope::Current, source)
        .expect("read request remains live");
    let counters = runtime.performance_access().counters();

    assert_eq!(neighbors.outgoing_relation_ids, vec![relation]);
    assert!(neighbors.incoming_relation_ids.is_empty());
    assert_eq!(
        counters.visible_authoritative_relation_records_materialized,
        0
    );
    assert_eq!(counters.visibility_relation_slot_scans, 0);
}

#[test]
fn complexity_budget_commit_inspection_reads_are_index_explicit_and_bounded() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _first = create_entity(execution, &runtime, "first");
    let _second = create_entity(execution, &runtime, "second");
    let latest_commit = runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id)
        .expect("latest commit");

    runtime.performance_access().reset_counters();
    let commit = runtime
        .inspect_what_happened()
        .inspect_commit(latest_commit)
        .expect("commit inspection");
    let recent =
        runtime
            .inspect_what_happened()
            .inspect_recent_commits(&RecentCommitInspectionRequest {
                branch_id: Some(crate::facade::history::BranchId("main".to_string())),
                limit: 2,
            });
    let counters = runtime.performance_access().counters();

    assert_eq!(commit.commit.commit_id, latest_commit);
    assert_eq!(recent.commits.len(), 2);
    assert_eq!(counters.inspection_commit_reads, 3);
    assert_eq!(
        counters.visible_authoritative_entity_records_materialized,
        0
    );
    assert_eq!(
        counters.visible_authoritative_relation_records_materialized,
        0
    );
}
