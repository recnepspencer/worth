//! #277 Native half, the re-delete: a Preserve round deletes a record that
//! existed before its chain. A successor round deletes it again. Native
//! proves that delete against the chain's own and folds it to nothing, so the
//! commit equals the one-round answer. Any other delete or write of that
//! record refuses typed, publishing nothing.
use super::*;

const LINK: KindId = KindId(2);

/// What one round does to the record, beside writing its output.
type Effect = fn(EntityId) -> MutationIntent;
const KEEP: &[Effect] = &[];
const DELETE: &[Effect] = &[delete];
const PATCH: &[Effect] = &[patch];

fn delete(entity_id: EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
        entity_id,
    }))
}
fn patch(entity_id: EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id,
            fields: string_aspect_field_patch([(aspect_key("name"), field_key("name"), "patch")]),
        },
    ))
}

/// A world whose one relation kind is deleted with an endpoint (`cascading`)
/// or retained dangling for audit.
fn world(execution: &ExecutionRequest<'_, '_>, cascading: bool) -> World {
    if cascading {
        World::new_cascading(execution)
    } else {
        World::new(execution, true, true)
    }
}

/// A committed record no route owns, linked (fieldless, kind 2) to the
/// world's carrier. Returns the record and its link.
fn linked_record(world: &World, execution: &ExecutionRequest<'_, '_>) -> (EntityId, RelationId) {
    let key = "doomed";
    let link = MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: LINK,
        client_key: ClientKey::raw("link:doomed"),
        source: EntityReference::Created(target(key)),
        target: EntityReference::Existing(world.carrier),
        fields: AspectFieldPatch::default(),
    }));
    let mut tx = begin_main_transaction(&world.runtime);
    tx.push_batch(create(key).push(link), execution.allocation_policy())
        .unwrap();
    let result = tx.commit(execution, &world.runtime).unwrap();
    let record = result.created_entity(&target(key)).unwrap();
    release_test_commit_snapshot(&world.runtime, &result);
    let links = links_into_carrier(world, execution);
    assert_eq!(links.len(), 1, "the record's one link");
    (record, links[0])
}

/// The kind-2 relations into the world's carrier, as a committed read lists them.
fn links_into_carrier(world: &World, execution: &ExecutionRequest<'_, '_>) -> Vec<RelationId> {
    world
        .runtime
        .read_truth()
        .project_observation(&world.basis().observation())
        .unwrap()
        .bounded_incoming_relations_of_kind(execution, world.carrier, LINK, None)
        .unwrap()
        .expect("no traversal quota")
        .into_records()
        .iter()
        .map(|record| record.relation_id)
        .collect()
}

/// Whether a committed read still finds the record, and its link.
fn reads(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    (record, link): (EntityId, RelationId),
) -> (bool, bool) {
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&world.basis().observation())
        .unwrap();
    let read = world
        .runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    let found = (
        read.get_entity(record).is_some(),
        read.get_relation(link).is_some(),
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
    found
}

/// One Preserve chain over `owned-output` in a fresh world. Each entry is one
/// round: the value it writes on the output, and what it does to the linked
/// record. The first is appended, the rest are successors. Returns the commit
/// with whether a committed read still finds the record's link, or the
/// refusal, having checked that it published nothing.
fn chain(cascading: bool, rounds: &[(&str, &[Effect])]) -> Result<(Observed, bool), Cause> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = world(execution, cascading);
    let prior = world.birth(execution, "owned", "owned-output");
    let (record, link) = linked_record(&world, execution);
    let meaning = preserve_meaning(&prior);
    let before = world.basis();
    let round = |&(value, effects): &(&str, &[Effect])| {
        vec![effects
            .iter()
            .fold(update(output(&prior), value), |batch, effect| {
                batch.push(effect(record))
            })]
    };
    let prepared = world.prepare(
        execution,
        &assemble(|execution, candidate| {
            candidate.append_publication(
                execution,
                preserve(&world, &prior),
                &none(),
                round(&rounds[0]),
            )?;
            for later in &rounds[1..] {
                successor(execution, candidate, preserve(&world, &prior), round(later))?;
            }
            Ok(())
        }),
    );
    match prepared {
        Ok(proposal) => {
            let observed = committed(&world, execution, proposal, &meaning).0;
            let (record, link) = reads(&world, execution, (record, link));
            assert!(!record, "the committed chain deleted the record");
            Ok((observed, link))
        }
        Err(err) => {
            assert_eq!(world.basis(), before, "nothing published");
            Err(refusal(err))
        }
    }
}

/// The head round deletes a record that existed before the chain, with a
/// link of each cascade policy in turn; the successor deletes it again. The
/// commit is the world where the successor's round is the only one: the same
/// states, patches and touches, and the same link. A third round folds the
/// same way.
#[test]
fn a_successor_re_deleting_the_record_its_chain_deleted_folds_to_the_one_round_world() {
    let mut worlds = Vec::new();
    for cascading in [false, true] {
        let one = chain(cascading, &[("round-3", DELETE)]).unwrap();
        assert_eq!(one.1, !cascading, "a retained link is still read");
        let two = chain(cascading, &[("round-2", DELETE), ("round-3", DELETE)]);
        assert_eq!(two.unwrap(), one, "two rounds, cascading={cascading}");
        let three = chain(
            cascading,
            &[
                ("round-1", DELETE),
                ("round-2", DELETE),
                ("round-3", DELETE),
            ],
        );
        assert_eq!(three.unwrap(), one, "three rounds, cascading={cascading}");
        worlds.push(one);
    }
    assert_ne!(worlds[0], worlds[1], "each policy writes its own records");
}

/// A successor that keeps a record its chain deleted leaves that delete
/// uncovered: the fold cannot restore the record.
#[test]
fn a_successor_keeping_the_record_its_chain_deleted_is_uncovered() {
    let refused = chain(false, &[("round-1", DELETE), ("round-2", KEEP)]);
    assert_eq!(refused.unwrap_err(), postponed(Reason::UncoveredWrite));
}

/// A successor that writes a record its chain deleted, without deleting it
/// again, leaves the delete uncovered too.
#[test]
fn a_successor_writing_the_record_its_chain_deleted_is_uncovered() {
    let refused = chain(false, &[("round-1", DELETE), ("round-2", PATCH)]);
    assert_eq!(refused.unwrap_err(), postponed(Reason::UncoveredWrite));
}

/// The re-delete is the record's one write in the round. Deleting it twice,
/// or deleting and writing it in either order, is not one round's answer.
#[test]
fn a_successor_re_deleting_a_record_beside_another_write_of_it_is_a_structural_effect() {
    let seconds: [&[Effect]; 3] = [&[delete, delete], &[patch, delete], &[delete, patch]];
    for second in seconds {
        let refused = chain(false, &[("round-1", DELETE), ("round-2", second)]);
        assert_eq!(refused.unwrap_err(), postponed(Reason::StructuralEffect));
    }
}

/// A successor that deletes a record its chain kept makes a new effect: the
/// fold may not drop a delete no earlier round applied.
#[test]
fn a_successor_deleting_a_record_its_chain_kept_is_a_structural_effect() {
    let refused = chain(false, &[("round-1", KEEP), ("round-2", DELETE)]);
    assert_eq!(refused.unwrap_err(), postponed(Reason::StructuralEffect));
}

/// The chain's delete retained the record's link, a record the chain changed.
/// No other writer can change that link afterwards: Native refuses a later
/// stage's write of it as a stale target. A round is therefore never judged
/// against a link another writer changed behind its chain's delete.
#[test]
fn another_writer_cannot_change_the_link_a_chains_delete_retained() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = world(execution, false);
    let prior = world.birth(execution, "owned", "owned-output");
    let (record, relation_id) = linked_record(&world, execution);
    let before = world.basis();
    let err = world
        .prepare_stages(
            execution,
            &[world.carrier],
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    preserve(&world, &prior),
                    &none(),
                    vec![update(output(&prior), "round-1").push(delete(record))],
                )?;
                let unlink = MutationIntent::Relation(RelationMutationIntent::Delete(
                    DeleteRelationIntent { relation_id },
                ));
                candidate.append_authored_stage(
                    execution,
                    world.carrier,
                    &none(),
                    vec![update(world.carrier, "stage-carrier").push(unlink)],
                )?;
                Ok(())
            }),
        )
        .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = err else {
        panic!("a stale-target conflict: {err:?}")
    };
    assert!(
        matches!(
            &error.class,
            ConflictClass::StaleTarget { target: ExistingRecordTarget::Relation(id), .. }
                if *id == relation_id
        ),
        "{error:?}"
    );
    assert_eq!(world.basis(), before);
}
