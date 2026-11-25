//! C-3: a successor round's relation effects over a shared anchor (#119 §2.4).
use super::*;

/// The kind-`LINK` relations into the carrier, as a committed read lists them.
fn into_carrier(world: &World) -> Vec<RelationId> {
    into_carrier_at(world, &world.basis())
}

/// `into_carrier` as of `basis`; an earlier basis reads the anchor's
/// historical kind bucket.
fn into_carrier_at(world: &World, basis: &AdmittedRelationalBranchBasis) -> Vec<RelationId> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let mut ids = world
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap()
        .bounded_incoming_relations_of_kind(execution, world.carrier, LINK, None)
        .unwrap()
        .expect("no traversal quota")
        .into_records()
        .iter()
        .map(|record| record.relation_id)
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

/// The chain links its output into the carrier; a stage links the source
/// into the same carrier between the rounds; the last round keeps or drops
/// its link. The stage's link survives either way, the chain's dropped link
/// is gone from the anchor, and the images equal the reference answer.
#[test]
fn a_dropped_link_leaves_a_shared_anchor_with_only_live_links() {
    let assembly = |rounds: &'static [bool]| {
        move |world: &World, execution: &ExecutionRequest<'_, '_>, candidate: &mut Candidate<'_>| {
            let linked = |keep: bool| {
                let mut batches = creates(&["x"], if keep { "kept" } else { "dropped" });
                if keep {
                    batches.push(WorkerIntentBatch::new("link").push(link(
                        "x-link",
                        EntityReference::Created(target("x")),
                        EntityReference::Existing(world.carrier),
                    )));
                }
                batches
            };
            candidate.append_publication(
                execution,
                birth(world, execution, &["x"]),
                &none(),
                linked(rounds[0]),
            )?;
            let stage = update(world.carrier, "stage-carrier").push(link(
                "stage-link",
                EntityReference::Existing(world.source),
                EntityReference::Existing(world.carrier),
            ));
            candidate.append_authored_stage(execution, world.carrier, &none(), vec![stage])?;
            for &keep in &rounds[1..] {
                successor(
                    execution,
                    candidate,
                    birth(world, execution, &["x"]),
                    linked(keep),
                )?;
            }
            Ok(())
        }
    };
    for last in [false, true] {
        let rounds: &'static [bool] = if last { &[true, true] } else { &[true, false] };
        let reference: &'static [bool] = if last { &[true] } else { &[false] };
        let chain = run(true, &assembly(rounds)).unwrap();
        let one = run(true, &assembly(reference)).unwrap();
        assert_eq!(chain.observed, one.observed, "keep={last}");
        assert_eq!(
            into_carrier(&chain.world).len(),
            if last { 2 } else { 1 },
            "keep={last}"
        );
        assert_eq!(
            into_carrier(&chain.world).len(),
            into_carrier(&one.world).len()
        );
        // The checkpoint captures every anchor's stored adjacency: the
        // reopened world reads exactly what the live one does.
        let host = request();
        let execution = &ExecutionRequest::serial(&host);
        let (cold, _) = cold::reopen(&chain.world, execution, true);
        let reopened = World {
            runtime: cold,
            ..run(true, &assembly(rounds)).unwrap().world
        };
        assert_eq!(
            into_carrier(&reopened),
            into_carrier(&chain.world),
            "keep={last}"
        );
        assert_eq!(
            states(&cold::facts(
                &reopened.runtime,
                execution,
                &result_ids(&chain.world, &chain.outputs)
            )),
            chain.observed.states
        );
    }
}

/// A committed `LINK` from the source into the carrier and a spare entity.
fn linked_world(world: World) -> (World, RelationId, EntityId) {
    let (world, spare) = super::replacement::spare(world);
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let mut tx = begin_main_transaction(&world.runtime);
    let base = link(
        "base-link",
        EntityReference::Existing(world.source),
        EntityReference::Existing(world.carrier),
    );
    tx.push_batch(
        WorkerIntentBatch::new("link").push(base),
        execution.allocation_policy(),
    )
    .unwrap();
    let result = tx.commit(execution, &world.runtime).unwrap();
    release_test_commit_snapshot(&world.runtime, &result);
    let relation = into_carrier(&world)[0];
    (world, relation, spare)
}

/// Owned Birth rounds each creating `x`; a round whose flag is set moves
/// the committed link's source to the spare entity, keeping the carrier as
/// its target.
fn endpoint_moves(rounds: &[bool]) -> (Run, AdmittedRelationalBranchBasis) {
    let ids = std::cell::Cell::new(None);
    let before = RefCell::new(None);
    let setup = |world: World| {
        let (world, relation, spare) = linked_world(world);
        ids.set(Some((relation, spare)));
        *before.borrow_mut() = Some(world.basis());
        world
    };
    let run = run_in(setup, false, &|world, execution, candidate| {
        let (relation, spare) = ids.get().expect("the link is committed");
        for (index, &moves) in rounds.iter().enumerate() {
            let mut round = creates(&["x"], "1");
            if moves {
                let to = UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: LINK,
                    source: EntityReference::Existing(spare),
                    target: EntityReference::Existing(world.carrier),
                };
                round.push(
                    WorkerIntentBatch::new("move").push(MutationIntent::Relation(
                        RelationMutationIntent::UpdateEndpoints(to),
                    )),
                );
            }
            let posture = birth(world, execution, &["x"]);
            if index == 0 {
                candidate.append_publication(execution, posture, &none(), round)?;
            } else {
                successor(execution, candidate, posture, round)?;
            }
        }
        Ok(())
    })
    .unwrap();
    (run, before.into_inner().expect("the setup basis"))
}

/// Round 1 moves a committed link's source and round 2 omits or repeats the
/// move. The carrier (the shared anchor) had the link in its historical
/// bucket before the chain, so retracting the move restores its current link
/// and keeps that history: a read as of the pre-chain basis still finds it,
/// and the result's checkpoint reopens and reads like the one-round answer.
#[test]
fn a_retracted_endpoint_move_keeps_the_shared_anchors_history() {
    for last in [false, true] {
        let (chain, before) = endpoint_moves(&[true, last]);
        let (one, one_before) = endpoint_moves(&[last]);
        assert_eq!(chain.observed.alpha(), one.observed.alpha(), "last={last}");
        assert_eq!(into_carrier(&chain.world), into_carrier(&one.world));
        let historical = into_carrier_at(&chain.world, &before);
        assert_eq!(historical.len(), 1, "last={last}: the pre-chain link");
        assert_eq!(historical, into_carrier_at(&one.world, &one_before));
        let host = request();
        let execution = &ExecutionRequest::serial(&host);
        let (cold, _) = cold::reopen(&chain.world, execution, true);
        let reopened = World {
            runtime: cold,
            ..endpoint_moves(&[true, last]).0.world
        };
        assert_eq!(
            into_carrier(&reopened),
            into_carrier(&one.world),
            "last={last}"
        );
    }
}
