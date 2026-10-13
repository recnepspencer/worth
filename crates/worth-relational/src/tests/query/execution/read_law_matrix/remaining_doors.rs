//! Answers and admission at the remaining exact read doors.
use super::*;
use crate::facade::indexes::{
    BoundedEntityFieldSelectedLookup, BoundedIndexParityMode, BoundedRelationJoinLookupRequest,
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexGenerationId, DerivedIndexKind,
    RelationJoinDefinition, RelationJoinLeg, RelationJoinSharedEndpoint,
};

fn with_backings(mut run: impl FnMut(&ExecutionRequest<'_, '_>)) {
    let host = crate::test_execution_policy::serial_request();
    let lease = test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), MEMORY, 100_000),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    run(&ExecutionRequest::serial(&host));
    run(&ExecutionRequest::leased(&lease));
}

#[test]
fn runtime_read_law_remaining_view_input_preparation() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = fixture::World::new(&setup, true);
    let view = world
        .runtime
        .read_truth()
        .project_snapshot(&world.snapshot)
        .unwrap();
    for parity in [
        BoundedIndexParityMode::Production,
        BoundedIndexParityMode::Certification,
    ] {
        with_backings(|execution| {
            let answer = {
                let prepared = world
                    .runtime
                    .index_access()
                    .prepare_entity_field_lookup(
                        &view,
                        world.index,
                        KindId(1),
                        &world.name,
                        execution,
                    )
                    .unwrap();
                world
                    .runtime
                    .index_access()
                    .execute_prepared_entity_field_lookup(
                        &prepared,
                        &string_aspect_value("a"),
                        2,
                        parity,
                        execution,
                        || Ok::<_, ()>(()),
                    )
                    .unwrap()
            };
            assert_eq!(answer.candidate_entity_ids(), &[world.entities[0]]);
            assert_eq!(answer.generation_id(), DerivedIndexGenerationId(1));
            assert_eq!(answer.examined_entry_count(), 1);
            assert!(!answer.overflowed());
        });
    }
}

#[test]
fn runtime_read_law_remaining_selected_view() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = fixture::World::new(&setup, true);
    let view = world
        .runtime
        .read_truth()
        .project_snapshot(&world.snapshot)
        .unwrap();
    let value = string_aspect_value("a");
    with_backings(|execution| {
        let mut charges = Vec::new();
        let answer = {
            world
                .runtime
                .index_access()
                .execute_bounded_entity_field_lookup(
                    BoundedEntityFieldSelectedLookup::new(
                        &view,
                        world.index,
                        KindId(1),
                        &world.name,
                        &value,
                        2,
                        |unit, bytes| {
                            charges.push((unit, bytes));
                            Ok::<_, ()>(())
                        },
                    ),
                    BoundedIndexParityMode::Production,
                    execution,
                )
                .unwrap()
        };
        assert_eq!(answer.candidate_entity_ids(), &[world.entities[0]]);
        assert_eq!(answer.generation_id(), DerivedIndexGenerationId(1));
        assert_eq!(answer.examined_entry_count(), 1);
        assert!(!answer.overflowed());
        assert!(
            !charges.is_empty(),
            "the selected lookup consults its caller's admission"
        );
    });
}

#[test]
fn runtime_read_law_remaining_relation_join() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = fixture::World::new(&setup, true);
    let index = world
        .runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: crate::facade::indexes::DerivedIndexId(90),
            name: "read-law-join".into(),
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
                KindId(1),
            )),
            branch_scoped: false,
        });
    let built = world.runtime.index_authority().build_for_commit(
        &setup,
        DerivedIndexBuildRequest {
            source_commit_id: world.runtime.history().latest_commit().unwrap().commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(built.failed_indexes.is_empty());
    for parity in [
        BoundedIndexParityMode::Production,
        BoundedIndexParityMode::Certification,
    ] {
        with_backings(|execution| {
            let request = BoundedRelationJoinLookupRequest::new(
                world.snapshot.clone(),
                index.index_id,
                world.entities[0],
                world.entities[2],
                2,
            )
            .unwrap();
            let answer = {
                world
                    .runtime
                    .index_access()
                    .execute_bounded_relation_join_lookup(request, parity, execution)
                    .unwrap()
            };
            assert_eq!(answer.candidate_entity_ids(), &[world.entities[1]]);
            assert_eq!(answer.generation_id(), DerivedIndexGenerationId(2));
            assert_eq!(answer.examined_entry_count(), 1);
            assert_eq!(answer.verified_entity_record_count(), 3);
            assert_eq!(answer.verified_relation_record_count(), 2);
            assert!(!answer.overflowed());
        });
    }
}
