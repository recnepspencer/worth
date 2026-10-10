use super::*;
use crate::facade::indexes::{
    BoundedIndexParityMode, BoundedRelationJoinLookupDenialKind, BoundedRelationJoinLookupRequest,
    DerivedIndexEntries, RelationJoinDefinition, RelationJoinEntry, RelationJoinKey,
    RelationJoinLeg, RelationJoinSharedEndpoint,
};

#[test]
fn relation_join_isolates_both_exact_endpoints_and_certifies_storage_parity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let left = create_entity(&runtime, "left");
    let right = create_entity(&runtime, "right");
    let other_left = create_entity(&runtime, "other-left");
    let other_right = create_entity(&runtime, "other-right");
    let selected = create_entity(&runtime, "selected");
    let same_left = create_entity(&runtime, "same-left");
    let same_right = create_entity(&runtime, "same-right");
    bind_chain(&runtime, left, selected, right, "selected");
    bind_chain(&runtime, left, same_left, other_right, "same-left");
    bind_chain(&runtime, other_left, same_right, right, "same-right");
    let index = register_relation_join(&runtime, 81);
    build_current_generation(execution, &runtime, index.index_id);
    let snapshot = runtime.visibility_authority().snapshot();
    let request = || join_request(snapshot.clone(), index.index_id, left, right, 2);

    let production = runtime
        .index_access()
        .execute_bounded_relation_join_lookup(
            request(),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap();
    let certified = runtime
        .index_access()
        .execute_bounded_relation_join_lookup(
            request(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();

    assert_eq!(production.candidate_entity_ids(), &[selected]);
    assert_eq!(production.examined_entry_count(), 1);
    assert_eq!(production.verified_entity_record_count(), 3);
    assert_eq!(production.verified_relation_record_count(), 2);
    assert!(!production.overflowed());
    assert_eq!(
        production.candidate_entity_ids(),
        certified.candidate_entity_ids()
    );
    assert_eq!(production.overflowed(), certified.overflowed());
}

#[test]
fn relation_join_certification_rejects_a_missing_candidate() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let left = create_entity(&runtime, "left");
    let right = create_entity(&runtime, "right");
    let selected = create_entity(&runtime, "selected");
    bind_chain(&runtime, left, selected, right, "selected");
    let index = register_relation_join(&runtime, 84);
    build_current_generation(execution, &runtime, index.index_id);
    let key = RelationJoinKey::new(left, right);
    runtime
        .indexes
        .corrupt_latest_generation(index.index_id, |generation| {
            let DerivedIndexEntries::RelationJoin(entries) = &mut generation.entries else {
                panic!("relation join generation expected");
            };
            entries.remove(&key);
        });
    let snapshot = runtime.visibility_authority().snapshot();

    let denial = runtime
        .index_access()
        .execute_bounded_relation_join_lookup(
            join_request(snapshot, index.index_id, left, right, 2),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap_err();

    assert_eq!(
        denial.kind(),
        BoundedRelationJoinLookupDenialKind::StorageParityMismatch
    );
}

#[test]
fn relation_join_requires_an_exact_generation_and_reports_bounded_overflow() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let left = create_entity(&runtime, "left");
    let right = create_entity(&runtime, "right");
    let first = create_entity(&runtime, "first");
    bind_chain(&runtime, left, first, right, "first");
    let index = register_relation_join(&runtime, 82);
    build_current_generation(execution, &runtime, index.index_id);

    let second = create_entity(&runtime, "second");
    bind_chain(&runtime, left, second, right, "second");
    let current = runtime.visibility_authority().snapshot();
    let missing = runtime
        .index_access()
        .execute_bounded_relation_join_lookup(
            join_request(current.clone(), index.index_id, left, right, 1),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap_err();
    assert_eq!(
        missing.kind(),
        BoundedRelationJoinLookupDenialKind::ExactGenerationUnavailable
    );

    build_current_generation(execution, &runtime, index.index_id);
    let bounded = runtime
        .index_access()
        .execute_bounded_relation_join_lookup(
            join_request(current, index.index_id, left, right, 1),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    assert_eq!(bounded.candidate_entity_ids(), &[first]);
    assert_eq!(bounded.examined_entry_count(), 1);
    assert!(bounded.overflowed());
}

#[test]
fn relation_join_rejects_a_candidate_with_substituted_relation_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let left = create_entity(&runtime, "left");
    let right = create_entity(&runtime, "right");
    let selected = create_entity(&runtime, "selected");
    let other_left = create_entity(&runtime, "other-left");
    let other_right = create_entity(&runtime, "other-right");
    let other_shared = create_entity(&runtime, "other-shared");
    let (selected_left_relation, _) = bind_chain(&runtime, left, selected, right, "selected");
    let (_, substituted_right_relation) =
        bind_chain(&runtime, other_left, other_shared, other_right, "other");
    let index = register_relation_join(&runtime, 83);
    build_current_generation(execution, &runtime, index.index_id);
    let key = RelationJoinKey::new(left, right);
    runtime
        .indexes
        .corrupt_latest_generation(index.index_id, |generation| {
            let DerivedIndexEntries::RelationJoin(entries) = &mut generation.entries else {
                panic!("relation join generation expected");
            };
            entries.get_mut(&key).unwrap()[0] = RelationJoinEntry::new(
                selected,
                selected_left_relation,
                substituted_right_relation,
            );
        });
    let snapshot = runtime.visibility_authority().snapshot();

    let denial = runtime
        .index_access()
        .execute_bounded_relation_join_lookup(
            join_request(snapshot, index.index_id, left, right, 2),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap_err();

    assert_eq!(
        denial.kind(),
        BoundedRelationJoinLookupDenialKind::CorruptIndexEntries
    );
}

#[test]
fn exact_relation_join_uses_one_branch_root_after_both_heads_diverge() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let left = create_entity(&runtime, "exact-left");
    let right = create_entity(&runtime, "exact-right");
    let selected = create_entity(&runtime, "exact-selected");
    let (selected_left_relation, _) = bind_chain(&runtime, left, selected, right, "exact-selected");
    let index = register_relation_join_with_scope(&runtime, 85, true);
    build_current_generation(execution, &runtime, index.index_id);
    let main_snapshot = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));

    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("join-sibling".to_owned()),
            &BranchId("main".to_owned()),
        )
        .unwrap();
    let sibling_snapshot =
        snapshot_for_owner_branch(&runtime, &BranchId("join-sibling".to_owned()));
    create_entity_outcome_on_branch(
        &runtime,
        "sibling-after-observation",
        BranchId("join-sibling".to_owned()),
    );
    delete_relation_on_branch(
        &runtime,
        selected_left_relation,
        BranchId("main".to_owned()),
    );
    build_current_generation(execution, &runtime, index.index_id);
    let current_main = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));

    let current_outcome = runtime
        .index_access()
        .execute_bounded_relation_join_lookup(
            join_request(current_main, index.index_id, left, right, 2),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    assert!(current_outcome.candidate_entity_ids().is_empty());

    for parity in [
        BoundedIndexParityMode::Production,
        BoundedIndexParityMode::Certification,
    ] {
        let outcome = runtime
            .index_access()
            .execute_bounded_relation_join_lookup(
                join_request(main_snapshot.clone(), index.index_id, left, right, 2),
                parity,
                execution,
            )
            .unwrap();
        assert_eq!(outcome.candidate_entity_ids(), &[selected]);
    }

    let sibling_denial = runtime
        .index_access()
        .execute_bounded_relation_join_lookup(
            join_request(sibling_snapshot, index.index_id, left, right, 2),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap_err();
    assert_eq!(
        sibling_denial.kind(),
        BoundedRelationJoinLookupDenialKind::ExactGenerationUnavailable
    );
}

fn register_relation_join(runtime: &RelationalRuntime, id: u64) -> DerivedIndexDefinition {
    register_relation_join_with_scope(runtime, id, false)
}

fn register_relation_join_with_scope(
    runtime: &RelationalRuntime,
    id: u64,
    branch_scoped: bool,
) -> DerivedIndexDefinition {
    runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(id),
        name: format!("relation.join.{id}"),
        kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
            RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
            RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
            KindId(1),
        )),
        branch_scoped,
    })
}

fn build_current_generation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    index_id: DerivedIndexId,
) {
    let source_commit_id = runtime.history().latest_commit().unwrap().commit_id;
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
}

fn bind_chain(
    runtime: &RelationalRuntime,
    left: crate::facade::identity::EntityId,
    shared: crate::facade::identity::EntityId,
    right: crate::facade::identity::EntityId,
    name: &str,
) -> (
    crate::facade::identity::RelationId,
    crate::facade::identity::RelationId,
) {
    let left_relation = create_relation(runtime, left, shared, &format!("{name}-left"));
    let right_relation = create_relation(runtime, shared, right, &format!("{name}-right"));
    (left_relation, right_relation)
}

fn join_request(
    snapshot: crate::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    left: crate::facade::identity::EntityId,
    right: crate::facade::identity::EntityId,
    limit: usize,
) -> BoundedRelationJoinLookupRequest {
    BoundedRelationJoinLookupRequest::new(snapshot, index_id, left, right, limit).unwrap()
}
