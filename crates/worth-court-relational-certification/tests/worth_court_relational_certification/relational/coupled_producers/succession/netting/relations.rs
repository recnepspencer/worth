//! C-3: a successor round's relation effects over a shared anchor (#119 §2.4).
use super::*;

/// The kind-`LINK` relations into the carrier, as a committed read lists them.
fn into_carrier(world: &World) -> Vec<RelationId> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let mut ids = world
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
