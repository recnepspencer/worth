//! Field/join preparation pays selected reads and deferred catalog geometry before effects.
use super::*;
use crate::facade::indexes::{RelationJoinDefinition, RelationJoinLeg, RelationJoinSharedEndpoint};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

#[test]
fn selected_field_and_join_candidate_keep_original_funding_until_performed_installation() {
    let memory = SerialMemoryBudget::new(64 * 1024 * 1024);
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = &ExecutionRequest::serial(&host);
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
        .index_id;
    let unrelated = index(&runtime);
    let request = DerivedIndexBuildRequest {
        source_commit_id: basis.observation().commit_id().unwrap(),
        branch_id: BranchId("main".into()),
        index_ids: vec![field, join],
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
    let mut measure = candidate_for_update(execution, &runtime, entity, "after");
    let measure_baseline = memory.charged_memory_bytes();
    let (prepared, report) = execution
        .run(ExecutionWorkCeiling::new(u64::MAX), |_| {
            runtime
                .index_authority()
                .prepare_incremental_indexes_for_candidate(
                    &mut measure,
                    &[field, join],
                    &before,
                    budget(0),
                    execution,
                )
        })
        .unwrap();
    prepared.unwrap();
    let required_work = report.charged_work();
    let required_memory = memory.charged_memory_bytes() - measure_baseline;
    assert!(required_work > 1);
    assert!(required_memory > 1);
    drop(measure);
    assert_eq!(memory.charged_memory_bytes(), initial_memory);

    let mut candidate = candidate_for_update(execution, &runtime, entity, "after");
    let candidate_baseline = memory.charged_memory_bytes();
    let commit = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    // Near-complete work refusal exercises the late quote on the SAME candidate.
    let (refused, _) = execution
        .run(ExecutionWorkCeiling::new(required_work - 1), |_| {
            runtime
                .index_authority()
                .prepare_incremental_indexes_for_candidate(
                    &mut candidate,
                    &[field, join],
                    &before,
                    budget(0),
                    execution,
                )
        })
        .unwrap();
    assert!(matches!(
        refused.unwrap_err().kind,
        DerivedIndexMaintenanceDenialKind::Execution(
            crate::execution::RelationalExecutionDenialCause::WorkExhausted
        )
    ));
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit)
        .is_empty());
    assert_eq!(memory.charged_memory_bytes(), candidate_baseline);
    let restricted_memory = SerialMemoryBudget::new(required_memory - 1);
    let unfunded = SerialRequest::from_memory(
        restricted_memory.clone(),
        CancellationSource::new().token(),
        None,
    );
    let memory_refusal = runtime
        .index_authority()
        .prepare_incremental_indexes_for_candidate(
            &mut candidate,
            &[field, join],
            &before,
            budget(0),
            &ExecutionRequest::serial(&unfunded),
        )
        .unwrap_err();
    assert!(matches!(
        memory_refusal.kind,
        DerivedIndexMaintenanceDenialKind::Execution(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
                | crate::execution::RelationalExecutionDenialCause::DeclaredMemoryExhausted { .. }
                | crate::execution::RelationalExecutionDenialCause::ProcessMemoryExhausted { .. }
        )
    ));
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit)
        .is_empty());
    assert_eq!(restricted_memory.charged_memory_bytes(), 0);

    let work = runtime
        .index_authority()
        .prepare_incremental_indexes_for_candidate(
            &mut candidate,
            &[field, join],
            &before,
            budget(0),
            execution,
        )
        .unwrap();
    assert_eq!(work.cold_record_slots, 0);
    assert!(work.patch_records > 0);
    assert_eq!(work.generation_publications_reserved, 2);
    assert_eq!(
        memory.charged_memory_bytes(),
        candidate_baseline + required_memory
    );
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
            .len(),
        2
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
        .is_none());
    assert_eq!(
        memory.charged_memory_bytes(),
        initial_memory,
        "performed installation discharges the original reservation"
    );
    runtime.snapshots().release_snapshot(&before).unwrap();
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}
