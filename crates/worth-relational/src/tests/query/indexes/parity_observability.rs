use super::*;

#[test]
fn derived_index_contract_failure_unknown_index_keeps_truth_reads_correct() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let outcome = create_entity_outcome(&runtime, "main-a");
    let snapshot = runtime.visibility_authority().snapshot();
    let storage_only = execute_explicit_query(
        execution,
        &runtime,
        &snapshot,
        "entities",
        vec![RecordRef::Entity(changed_entities(&outcome)[0])],
    )
    .result;
    let storage_read_before = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            planned_explicit_query(
                &runtime,
                &snapshot,
                "entities",
                vec![RecordRef::Entity(changed_entities(&outcome)[0])],
            ),
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .unwrap();
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![DerivedIndexId(999)],
        },
    );
    let storage_read_after = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            planned_explicit_query(
                &runtime,
                &snapshot,
                "entities",
                vec![RecordRef::Entity(changed_entities(&outcome)[0])],
            ),
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .unwrap();

    assert_eq!(build.failed_indexes, vec![DerivedIndexId(999)]);
    assert_eq!(
        storage_read_before.access_path,
        QueryAccessPath::AuthoritativeStorage
    );
    assert_eq!(storage_read_before.execution.result, storage_only);
    assert_eq!(storage_read_after.execution.result, storage_only);
}

#[test]
fn derived_index_contract_certification_mode_emits_stable_parity_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let outcome = create_entity_outcome(&runtime, "main-a");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.global".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
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
    let mut planned = crate::facade::query::PlannedQueryPacket::explicit_targets(
        "entities",
        context,
        vec![RecordRef::Entity(changed_entities(&outcome)[0])],
    );
    planned.access_contract = QueryAccessContract::DerivedIndexWithStorageParity;
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, planned)
        .expect("query plan");

    let first = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            plan.clone(),
            IndexParityMode::CertificationParity,
            execution,
        )
        .expect("request remains live")
        .expect("first parity outcome");
    let second = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::CertificationParity, execution)
        .expect("request remains live")
        .expect("second parity outcome");

    assert_eq!(first.access_path, second.access_path);
    assert_eq!(first.parity_basis_digest, second.parity_basis_digest);
}

#[test]
fn derived_index_contract_sampled_parity_is_bounded_and_deterministic() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(9),
        name: "entity.name.sampled".to_string(),
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
    let generation_id = build.generations[0].generation_id;
    let sampled_key = sampled_plan_key_for(generation_id, snapshot.version_id, true);
    let unsampled_key = sampled_plan_key_for(generation_id, snapshot.version_id, false);

    let sampled_packet = crate::facade::query::PlannedQueryPacket {
        label: "entity-name-sampled".to_string(),
        context_id: context.clone(),
        scope: crate::facade::query::QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("alpha"),
            partition_scope: None,
        },
        locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: sampled_key,
        target_count_hint: 0,
    };
    let unsampled_packet = crate::facade::query::PlannedQueryPacket {
        plan_key: unsampled_key,
        label: "entity-name-unsampled".to_string(),
        ..sampled_packet.clone()
    };

    runtime.performance_access().reset_counters();
    let sampled = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, sampled_packet.clone())
                .expect("sampled plan"),
            IndexParityMode::SampledParity,
            execution,
        )
        .expect("request remains live")
        .expect("sampled outcome");
    let sampled_repeat = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, sampled_packet)
                .expect("sampled plan repeat"),
            IndexParityMode::SampledParity,
            execution,
        )
        .expect("request remains live")
        .expect("sampled repeat outcome");
    let unsampled = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, unsampled_packet)
                .expect("unsampled plan"),
            IndexParityMode::SampledParity,
            execution,
        )
        .expect("request remains live")
        .expect("unsampled outcome");
    let counters = runtime.performance_access().counters();

    assert_eq!(sampled.access_path, sampled_repeat.access_path);
    assert_eq!(
        sampled.parity_basis_digest,
        sampled_repeat.parity_basis_digest
    );
    assert_eq!(
        sampled.access_path,
        QueryAccessPath::DerivedIndexGeneration { generation_id }
    );
    assert_eq!(
        unsampled.access_path,
        QueryAccessPath::DerivedIndexGeneration { generation_id }
    );
    assert_eq!(counters.query_index_attempt_count, 3);
    assert_eq!(counters.query_index_path_count, 3);
    assert_eq!(counters.query_index_parity_verification_count, 2);
}

#[test]
fn derived_index_contract_runtime_drop_releases_index_scratch_hints() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime_id;
    {
        let runtime = runtime_with_index_field_aspects();
        runtime_id = runtime.runtime_instance_id();
        let _alpha_a = create_entity_outcome(&runtime, "alpha");
        let _alpha_b = create_entity_outcome(&runtime, "alpha");
        let index = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(77),
            name: "entity.name.drop-release".to_string(),
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
        let packet = crate::facade::query::PlannedQueryPacket {
            label: "entity-name-drop-release".to_string(),
            context_id: context,
            scope: crate::facade::query::QueryScope::EntityFieldEquals {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                value: string_aspect_value("alpha"),
                partition_scope: None,
            },
            locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
            ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
            access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
            execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
            reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
            plan_key: crate::facade::query::DeterministicQueryPlanKey(2077),
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
        assert!(counters.query_index_scratch_reuse_count > 0);
        assert!(crate::indexes::index_query_scratch_hint_exists(runtime_id));
    }

    assert!(
        !crate::indexes::index_query_scratch_hint_exists(runtime_id),
        "runtime drop should release its own scratch hint even if other tests create hints concurrently"
    );
}

#[path = "parity_observability/path_counters.rs"]
mod path_counters;
