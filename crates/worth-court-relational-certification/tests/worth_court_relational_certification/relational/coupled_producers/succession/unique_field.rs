//! The #277 own-chain fold retires the exact prior unique-key holder.
use super::*;
use worth_relational::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use worth_relational::facade::transactions::InvariantViolationFields;
use worth_testkit_relational::aspect_storage::aspect_field_locator;

#[test]
fn a_coupled_successor_can_claim_a_key_its_own_chain_retires() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    for retire_holder in [false, true] {
        let world = World::new(execution, true, true);
        let prior = world.birth(execution, "owned", "owned-output");
        let meaning = preserve_meaning(&prior);
        let mut tx = begin_main_transaction(&world.runtime);
        tx.push_batch(create("held"), execution.allocation_policy())
            .unwrap();
        let committed_holder = tx.commit(execution, &world.runtime).unwrap();
        let holder = committed_holder.created_entity(&target("held")).unwrap();
        release_test_commit_snapshot(&world.runtime, &committed_holder);
        let basis = world.basis();
        let index = world
            .runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "node.name.unique".into(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
                branch_scoped: false,
                unique_entity_kind: Some(KindId(1)),
            })
            .unwrap();
        let generations = world.runtime.index_authority().build_for_basis(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().unwrap(),
                branch_id: basis.identity().branch_id().clone(),
                index_ids: vec![index.index_id],
            },
            &basis,
        );
        assert_eq!(generations.generations.len(), 1);

        let delete = || {
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: holder,
            }))
        };
        let round = |value| {
            let batch = update(output(&prior), value);
            vec![if retire_holder {
                batch.push(delete())
            } else {
                batch
            }]
        };
        let prepared = world.prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    preserve(&world, &prior),
                    &none(),
                    round("intermediate"),
                )?;
                successor(
                    execution,
                    candidate,
                    preserve(&world, &prior),
                    round("held"),
                )?;
                Ok(())
            }),
        );
        if retire_holder {
            committed(
                &world,
                execution,
                prepared.expect("the fold retires the incumbent"),
                &meaning,
            );
        } else {
            let Err(TransactionCommitError::Conflict { error, .. }) = prepared else {
                panic!("a surviving incumbent must deny the successor");
            };
            assert!(matches!(
                error.class,
                ConflictClass::InvariantViolation {
                    fields: InvariantViolationFields::UniqueEntityField {
                        entity_kind_id: KindId(1),
                        ..
                    },
                    ..
                }
            ));
            assert_eq!(world.basis(), basis, "the refused chain publishes nothing");
        }
    }
}
