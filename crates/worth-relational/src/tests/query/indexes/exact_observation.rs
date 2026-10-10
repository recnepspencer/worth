use super::*;

#[test]
fn index_query_keeps_the_admitted_root_after_same_and_sibling_reference_movement() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let alpha_id = changed_entities(&alpha)[0];
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.exact-observation".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
    );
    let snapshot = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));

    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("index-sibling".to_owned()),
            &BranchId("main".to_owned()),
        )
        .expect("sibling begins at the observed root");
    update_entity(&runtime, alpha_id, "main-new");
    create_entity_outcome_on_branch(
        &runtime,
        "sibling-only",
        BranchId("index-sibling".to_owned()),
    );

    runtime.performance_access().reset_counters();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("admitted snapshot retains its exact planning context");
    let packet = crate::facade::query::PlannedQueryPacket {
        label: "exact-observation-index".to_owned(),
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
        plan_key: crate::facade::query::DeterministicQueryPlanKey(9_170_601),
        target_count_hint: 1,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("exact observation plans after both references move");
    let authoritative = runtime
        .read_truth()
        .execute_query_plan(
            plan.clone(),
            execution,
            crate::facade::runtime::RelationalQueryReadStrategy::CallingThread,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("authoritative exact-root query succeeds");
    let indexed = runtime
        .index_access()
        .execute_query_plan_with_index_parity(plan, IndexParityMode::CertificationParity, execution)
        .expect("request remains live")
        .expect("indexed exact-root query succeeds");

    assert!(build.failed_indexes.is_empty());
    assert_eq!(indexed.execution.result, authoritative.result);
    assert_eq!(indexed.execution.result.entities.len(), 1);
    assert_eq!(indexed.execution.result.entities[0].entity_id, alpha_id);
    assert_eq!(
        indexed.access_path,
        QueryAccessPath::DerivedIndexGeneration {
            generation_id: build.generations[0].generation_id,
        }
    );
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.visibility_cache_miss_reconstructions, 0);
}

#[test]
fn current_query_prefers_its_exact_generation_over_a_later_historical_rebuild() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let alpha = create_entity_outcome(&runtime, "alpha");
    let alpha_id = changed_entities(&alpha)[0];
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.late-historical-rebuild".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let first = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
    );
    let current = update_entity(&runtime, alpha_id, "beta");
    let current_build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: current.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
    );
    let historical_rebuild = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
    );

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("current query context");
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            crate::facade::query::PlannedQueryPacket {
                label: "prefer-exact-current-generation".to_owned(),
                context_id: context,
                scope: crate::facade::query::QueryScope::EntityFieldEquals {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                    value: string_aspect_value("beta"),
                    partition_scope: None,
                },
                locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
                ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
                execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
                reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
                plan_key: crate::facade::query::DeterministicQueryPlanKey(9_170_602),
                target_count_hint: 1,
            },
        )
        .expect("current query plans");
    let indexed = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            plan,
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .expect("current indexed query executes");

    assert!(first.failed_indexes.is_empty());
    assert!(current_build.failed_indexes.is_empty());
    assert!(historical_rebuild.failed_indexes.is_empty());
    assert!(
        historical_rebuild.generations[0].generation_id
            > current_build.generations[0].generation_id,
        "the court requires the historical generation to be published later"
    );
    assert_eq!(
        indexed.access_path,
        QueryAccessPath::DerivedIndexGeneration {
            generation_id: current_build.generations[0].generation_id,
        }
    );
    assert_eq!(indexed.execution.result.entities.len(), 1);
    assert_eq!(indexed.execution.result.entities[0].entity_id, alpha_id);
}

#[test]
fn current_query_rejects_an_older_generation_and_returns_every_exact_match() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "shared-name");
    let first_id = changed_entities(&first)[0];
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.no-stale-generation".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let older_build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
    );
    let second = create_entity_outcome(&runtime, "shared-name");
    let second_id = changed_entities(&second)[0];

    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("current query context");
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            crate::facade::query::PlannedQueryPacket {
                label: "reject-older-index-generation".to_owned(),
                context_id: context,
                scope: crate::facade::query::QueryScope::EntityFieldEquals {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                    value: string_aspect_value("shared-name"),
                    partition_scope: None,
                },
                locality: crate::facade::query::QueryLocalityClass::CrossPartitionTraversal,
                ordering: crate::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
                execution_shape: crate::facade::query::QueryExecutionShape::BulkPacketized,
                reduction: crate::facade::query::ReductionDiscipline::DeterministicMerge,
                plan_key: crate::facade::query::DeterministicQueryPlanKey(9_170_603),
                target_count_hint: 2,
            },
        )
        .expect("current query plans");
    let outcome = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            plan,
            IndexParityMode::ProductionAdmissibility,
            execution,
        )
        .expect("request remains live")
        .expect("current query falls back to exact storage truth");

    assert!(older_build.failed_indexes.is_empty());
    assert!(older_build.generations[0].applicability.version_id < snapshot.version_id);
    assert_eq!(
        outcome.access_path,
        QueryAccessPath::DerivedIndexRejectedStorageRead {
            rejection: IndexQueryRejectionClass::UnsupportedVersion,
        }
    );
    assert_eq!(
        outcome
            .execution
            .result
            .entities
            .iter()
            .map(|entity| entity.entity_id)
            .collect::<Vec<_>>(),
        vec![first_id, second_id],
    );
}
