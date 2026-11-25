//! #277 Native half: a reopening chain's overlay exposes every relation its
//! rounds changed, with both live sides; and a Preserve round that unlinks a
//! relation folds like a relink: the successor must delete exactly what the
//! latest round deleted, and the commit equals the one-round answer.
use super::*;
use std::cell::RefCell;
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::mvcc::RelationalCoupledOverlayRelation as OverlayRelation;

const LINK: KindId = KindId(2);

/// The world's committed Birth of `owned-output`, linked (fieldless, kind 2)
/// to the world's source, as the Preserve rounds' prior publication.
fn linked_prior(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
) -> RelationalRequiredOutputPublication {
    let key = "owned-output";
    let mut tx = begin_main_transaction(&world.runtime);
    let link = MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: LINK,
        client_key: ClientKey::raw("link:owned-output"),
        source: EntityReference::Created(CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(key),
        }),
        target: EntityReference::Existing(world.source),
        fields: AspectFieldPatch::default(),
    }));
    tx.push_batch(create(key).push(link), execution.allocation_policy())
        .unwrap();
    let proposal = tx
        .validate(execution, &world.runtime)
        .unwrap()
        .bind_required_output_publication(
            &world.installed,
            RequiredOutputPublication::Birth {
                admission: world.owner.admit_required_output("owned", [1; 32]),
                absence: Some(world.absence(execution, "owned")),
                meaning: world.meaning("owned"),
                roles: vec![created(key)],
            },
        )
        .unwrap();
    let result = world.publish(execution, proposal);
    let publication = result
        .required_output_publication(&world.installed)
        .expect("real initial publication");
    release_test_commit_snapshot(&world.runtime, &result);
    publication
}

/// The live kind-2 relation out of `entity` in the candidate.
fn link_of(
    execution: &ExecutionRequest<'_, '_>,
    candidate: &Candidate<'_>,
    entity: EntityId,
) -> RelationId {
    let pin = candidate.pin_source(execution).unwrap();
    let records = pin
        .project_candidate()
        .bounded_outgoing_relations_of_kind(execution, entity, LINK, None)
        .unwrap()
        .expect("no traversal quota")
        .into_records();
    assert_eq!(records.len(), 1, "one link out of the output");
    records[0].relation_id
}

/// A Preserve round writing `value` and, when `unlink`, deleting `relation`.
fn preserve_round(
    output: EntityId,
    value: &str,
    unlink: Option<RelationId>,
) -> Vec<WorkerIntentBatch> {
    let batch = update(output, value);
    vec![match unlink {
        Some(relation_id) => batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent { relation_id },
        ))),
        None => batch,
    }]
}

/// One Preserve chain over a linked prior: each `(value, unlink)` is a round,
/// the first appended, the rest successors. Returns the commit and the chain's
/// overlay relations as seen before the last round.
fn preserve_chain(rounds: &[(&str, bool)]) -> (Result<Observed, Cause>, Vec<OverlayRelation>) {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = linked_prior(&world, execution);
    let meaning = preserve_meaning(&prior);
    let seen = RefCell::new(Vec::new());
    let prepared = world.prepare(
        execution,
        &assemble(|execution, candidate| {
            let relation = link_of(execution, candidate, output(&prior));
            let unlink = |on: bool| on.then_some(relation);
            let (value, on) = rounds[0];
            candidate.append_publication(
                execution,
                preserve(&world, &prior),
                &none(),
                preserve_round(output(&prior), value, unlink(on)),
            )?;
            for &(value, on) in &rounds[1..] {
                *seen.borrow_mut() = candidate.chain_overlay(execution, 0)?.relations().to_vec();
                successor(
                    execution,
                    candidate,
                    preserve(&world, &prior),
                    preserve_round(output(&prior), value, unlink(on)),
                )?;
            }
            Ok(())
        }),
    );
    let committed = match prepared {
        Ok(proposal) => Ok(committed(&world, execution, proposal, &meaning).0),
        Err(err) => Err(refusal(err)),
    };
    (committed, seen.into_inner())
}

/// A Birth round that links its output exposes a created overlay relation:
/// absent at the head pre-pin, live at current with the round's endpoints.
#[test]
fn a_chain_overlay_exposes_the_relation_its_birth_round_linked() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let keys = ["owned-output"];
    let seen = RefCell::new(Vec::new());
    world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    birth(&world, execution, &keys),
                    &none(),
                    rounds::linked_round(&world, &keys, "round-1", PartitionId::main(), &keys),
                )?;
                *seen.borrow_mut() = candidate.chain_overlay(execution, 0)?.relations().to_vec();
                Ok(())
            }),
        )
        .unwrap();
    let seen = seen.into_inner();
    assert_eq!(seen.len(), 1, "the round's one link");
    assert_eq!(seen[0].head(), None, "absent before the chain");
    let current = seen[0].current().expect("live after the round");
    assert_eq!(current.kind(), LINK);
    assert_eq!(current.target(), world.source);
}

/// A Preserve round that unlinks a relation exposes a deleted overlay
/// relation (live at the head pre-pin only), and a successor that deletes it
/// again folds: the commit equals the one-round answer and keeps one delete.
#[test]
fn a_preserve_round_unlinking_a_relation_folds() {
    let (folded, seen) = preserve_chain(&[("round-1", true), ("round-2", true)]);
    assert_eq!(seen.len(), 1, "the unlinked relation");
    assert_eq!(seen[0].current(), None, "deleted by the chain");
    assert_eq!(seen[0].head().expect("live before the chain").kind(), LINK);
    let (one, _) = preserve_chain(&[("round-2", true)]);
    assert_eq!(folded.unwrap(), one.unwrap());
}

/// A successor that keeps a relation its chain's latest round deleted leaves
/// that delete uncovered.
#[test]
fn a_successor_keeping_a_relation_its_chain_unlinked_is_uncovered() {
    let (refused, _) = preserve_chain(&[("round-1", true), ("round-2", false)]);
    assert_eq!(refused.unwrap_err(), postponed(Reason::UncoveredWrite));
}

/// A successor that deletes a relation its chain never deleted is a new
/// effect the fold cannot lower.
#[test]
fn a_successor_unlinking_a_relation_its_chain_kept_is_a_structural_effect() {
    let (refused, _) = preserve_chain(&[("round-1", false), ("round-2", true)]);
    assert_eq!(refused.unwrap_err(), postponed(Reason::StructuralEffect));
}

/// A round that moves a relation's endpoints exposes both live sides: the
/// head pre-pin endpoints and the moved current endpoints.
#[test]
fn a_moved_relation_exposes_both_live_sides() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = linked_prior(&world, execution);
    let seen = RefCell::new(Vec::new());
    world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                let relation_id = link_of(execution, candidate, output(&prior));
                let moved = update(output(&prior), "round-1").push(MutationIntent::Relation(
                    RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                        relation_id,
                        kind_id: LINK,
                        source: EntityReference::Existing(output(&prior)),
                        target: EntityReference::Existing(world.carrier),
                    }),
                ));
                candidate.append_publication(
                    execution,
                    preserve(&world, &prior),
                    &none(),
                    vec![moved],
                )?;
                *seen.borrow_mut() = candidate.chain_overlay(execution, 0)?.relations().to_vec();
                Ok(())
            }),
        )
        .unwrap();
    let seen = seen.into_inner();
    assert_eq!(seen.len(), 1, "the moved relation");
    assert_eq!(seen[0].head().expect("live before").target(), world.source);
    assert_eq!(
        seen[0].current().expect("live after").target(),
        world.carrier
    );
}
