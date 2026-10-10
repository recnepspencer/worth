use crate::facade::history::BranchId;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::facade::query::{
    DeterministicQueryPlanKey, IndexParityMode, PlannedQueryPacket, QueryAccessContract,
    QueryExecutionShape, QueryLocalityClass, QueryOrderingContract, QueryScope,
    ReductionDiscipline,
};
use crate::tests::support::*;

#[test]
fn complexity_budget_index_entity_field_equals_avoids_snapshot_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.lookup".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "entity-name-equals".to_string(),
        context_id: context,
        scope: QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("alpha"),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(2001),
        target_count_hint: 0,
    };

    runtime.performance_access().reset_counters();
    let _ = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet)
                .expect("query plan"),
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .expect("query outcome");
    let counters = runtime.performance_access().counters();

    assert_eq!(
        counters.visible_authoritative_entity_records_materialized,
        0
    );
    assert_eq!(counters.query_index_attempt_count, 1);
    assert_eq!(counters.query_index_path_count, 1);
    assert_eq!(counters.query_index_parity_verification_count, 0);
    assert_eq!(counters.query_authoritative_entity_records_emitted, 1);
}

#[test]
fn complexity_budget_index_relation_field_equals_avoids_snapshot_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity_outcome(&runtime, "source");
    let source_id = changed_entities(&source)[0];
    let target = create_entity_outcome(&runtime, "target");
    let target_id = changed_entities(&target)[0];
    let relation = create_relation_outcome(&runtime, source_id, target_id, "edge");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(1),
        name: "relation.label.lookup".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: relation.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "relation-label-equals".to_string(),
        context_id: context,
        scope: QueryScope::RelationFieldEquals {
            field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            value: string_aspect_value("edge"),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(2002),
        target_count_hint: 0,
    };

    runtime.performance_access().reset_counters();
    let _ = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet)
                .expect("query plan"),
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .expect("query outcome");
    let counters = runtime.performance_access().counters();

    assert_eq!(
        counters.visible_authoritative_relation_records_materialized,
        0
    );
    assert_eq!(counters.query_index_attempt_count, 1);
    assert_eq!(counters.query_index_path_count, 1);
    assert_eq!(counters.query_index_parity_verification_count, 0);
    assert_eq!(counters.query_authoritative_relation_records_emitted, 1);
}

#[test]
fn complexity_budget_index_field_equals_reuses_warm_index_scratch_on_repeated_lookup() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(2),
        name: "entity.name.reuse".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "entity-name-reuse".to_string(),
        context_id: context,
        scope: QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("alpha"),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(2003),
        target_count_hint: 0,
    };

    runtime.performance_access().reset_counters();
    for _ in 0..2 {
        let _ = runtime
            .index_access()
            .execute_query_plan_with_index_parity(
                runtime
                    .read_truth()
                    .plan_query_packet(&snapshot, packet.clone())
                    .expect("query plan"),
                IndexParityMode::ProductionAdmissibility,
                execution,
            )
            .expect("request remains live")
            .expect("query outcome");
    }
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.query_index_attempt_count, 2);
    assert_eq!(counters.query_index_path_count, 2);
    assert!(counters.query_index_scratch_reuse_count > 0);
}

#[test]
fn complexity_budget_index_field_equals_reports_actual_result_width() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _alpha_a = create_entity_outcome(&runtime, "alpha");
    let _alpha_b = create_entity_outcome(&runtime, "alpha");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(3),
        name: "entity.name.width".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let latest_commit_id = runtime
        .history()
        .latest_commit()
        .expect("latest commit")
        .commit_id;
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: latest_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "entity-name-width".to_string(),
        context_id: context,
        scope: QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("alpha"),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(2004),
        target_count_hint: 0,
    };

    runtime.performance_access().reset_counters();
    let outcome = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet)
                .expect("query plan"),
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .expect("query outcome");
    let counters = runtime.performance_access().counters();

    assert_eq!(outcome.execution.result.entities.len(), 2);
    assert_eq!(counters.query_packet_count, 1);
    assert_eq!(counters.query_packet_item_count, 2);
    assert_eq!(counters.query_packet_peak_width_total, 2);
}
