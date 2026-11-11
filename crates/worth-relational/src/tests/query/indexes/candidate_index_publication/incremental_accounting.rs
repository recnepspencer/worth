//! Field/join preparation pays selected reads and deferred catalog geometry before effects.
use super::*;
use crate::facade::indexes::{RelationJoinDefinition, RelationJoinLeg, RelationJoinSharedEndpoint};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

#[test]
fn selected_field_and_join_candidate_keep_original_funding_until_performed_installation() {
    let memory = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let maintenance_memory = SerialMemoryBudget::new(None);
    let maintenance_host = SerialRequest::from_memory(
        maintenance_memory.clone(),
        CancellationSource::new().token(),
        None,
    );
    let maintenance_execution = &ExecutionRequest::serial(&maintenance_host);
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&first)[0];
    let middle = create_entity(execution, &runtime, "middle");
    let right = create_entity(execution, &runtime, "right");
    create_relation(execution, &runtime, entity, middle, "left-leg");
    create_relation(execution, &runtime, middle, right, "right-leg");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let before = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let field = index(&runtime);
    let join = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "candidate.selected-join".into(),
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
                KindId(1),
            )),
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity")
        .index_id;
    let unrelated = index(&runtime);
    let request = DerivedIndexBuildRequest {
        source_commit_id: basis.observation().commit_id().unwrap(),
        branch_id: BranchId("main".into()),
        index_ids: vec![field, join, unrelated],
    };
    assert!(runtime
        .index_authority()
        .build_for_commit(execution, request)
        .failed_indexes
        .is_empty());
    let initial_join = runtime
        .indexes
        .published_generation_for_commit(
            join,
            Some(&BranchId("main".into())),
            basis.observation().commit_id().unwrap(),
            basis.observation().version_id(),
        )
        .unwrap();
    let DerivedIndexEntries::RelationJoin(rows) = &initial_join.entries else {
        panic!("native joined generation")
    };
    assert!(
        !rows.is_empty(),
        "genuine relation chain supplies joined payload"
    );

    // Measure the existing request report and retained reservation, not a new meter.
    // Dropping this unperformed candidate publishes neither source nor generations.
    let initial_memory = memory.charged_memory_bytes();
    let transaction = transaction_for_update(&runtime, entity, "after");
    let (prepared, report) = maintenance_execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            runtime.prepare_branch_transaction(maintenance_execution, transaction)
        })
        .unwrap();
    let measure = prepared.unwrap();
    let required_work = report.charged_work();
    let required_memory = maintenance_memory.charged_memory_bytes();
    assert!(required_work > 1);
    assert!(required_memory > 1);
    drop(measure);
    assert_eq!(memory.charged_memory_bytes(), initial_memory);
    assert_eq!(maintenance_memory.charged_memory_bytes(), 0);

    let transaction = transaction_for_update(&runtime, entity, "after");
    let (refused, _) = maintenance_execution
        .run(ExecutionWorkCeiling::new(Some(required_work - 1)), |_| {
            runtime.prepare_branch_transaction(maintenance_execution, transaction)
        })
        .unwrap();
    assert!(matches!(
        refused.unwrap_err(),
        crate::transactions::data::TransactionCommitError::Execution { .. }
    ));
    assert_eq!(maintenance_memory.charged_memory_bytes(), 0);
    let restricted_memory = SerialMemoryBudget::new(Some(required_memory - 1));
    let unfunded = SerialRequest::from_memory(
        restricted_memory.clone(),
        CancellationSource::new().token(),
        None,
    );
    let transaction = transaction_for_update(&runtime, entity, "after");
    let refused =
        runtime.prepare_branch_transaction(&ExecutionRequest::serial(&unfunded), transaction);
    assert!(matches!(
        refused.unwrap_err(),
        crate::transactions::data::TransactionCommitError::Execution { .. }
    ));
    assert_eq!(restricted_memory.charged_memory_bytes(), 0);

    let candidate = candidate_for_update(maintenance_execution, &runtime, entity, "after");
    let commit = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    let work = candidate.prepared_index_maintenance_work().unwrap();
    assert_eq!(work.cold_record_slots, 0);
    assert!(work.patch_records > 0);
    assert!(maintenance_memory.charged_memory_bytes() > 0);
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit)
        .is_empty());
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("genuine candidate performs")
    };
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit)
        .is_empty());
    let second = runtime.settle_performed_publication(performed).unwrap();
    assert_eq!(
        runtime
            .indexes
            .derived_artifacts_for_commit(commit)
            .generations()
            .iter()
            .map(|g| g.index_id)
            .collect::<Vec<_>>(),
        vec![field, unrelated]
    );
    assert_eq!(
        runtime
            .indexes
            .published_generation_for_commit(
                join,
                Some(&BranchId("main".into())),
                commit,
                second.commit.version_id
            )
            .unwrap()
            .generation_id,
        initial_join.generation_id
    );

    let generation = runtime
        .indexes
        .published_generation_for_commit(
            field,
            Some(&BranchId("main".into())),
            commit,
            second.commit.version_id,
        )
        .unwrap();
    let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
        panic!("declared field")
    };
    assert!(
        entries.contains_key(&AuthoritativeFieldComparisonKey::from_aspect_value(
            &string_aspect_value("after")
        ))
    );
    assert!(runtime
        .indexes
        .published_generation_for_commit(
            join,
            Some(&BranchId("main".into())),
            commit,
            second.commit.version_id
        )
        .is_some());
    assert!(runtime
        .indexes
        .published_generation_for_commit(
            unrelated,
            Some(&BranchId("main".into())),
            commit,
            second.commit.version_id
        )
        .is_some());
    assert!(
        maintenance_memory.charged_memory_bytes() > 0,
        "the original native payer funds canonical history too"
    );
    runtime.snapshots().release_snapshot(&before).unwrap();
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
    drop((
        runtime,
        initial_join,
        generation,
        first,
        second,
        basis,
        before,
    ));
    assert_eq!(memory.charged_memory_bytes(), 0);
    assert_eq!(maintenance_memory.charged_memory_bytes(), 0);
}
