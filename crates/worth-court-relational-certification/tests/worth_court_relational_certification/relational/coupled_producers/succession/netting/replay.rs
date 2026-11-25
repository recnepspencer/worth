//! C-15: coupled allocation evidence replays in the envelope's canonical plan
//! order. Live coupled allocation runs in member order, while the merged plan
//! sorts creates canonically (entities before relations, then by key), so the
//! two orders differ whenever members' keys sort out of member order.
use super::*;

/// An ordinary (unowned) producer part creating `key`: it writes records but
/// acquires no publication ownership, so its commit replays from the WAL.
fn ordinary_part(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    candidate: &mut Candidate<'_>,
    key: &str,
) -> Result<(), Failure> {
    candidate.append_publication(
        execution,
        RelationalCoupledProducerPosture::Ordinary {
            admission: world.owner.admit_required_output("ordinary", [1; 32]),
            absence: world.absence(execution, "ordinary"),
            meaning: world.meaning("ordinary"),
            roles: vec![created(key), existing(world.carrier)],
        },
        &none(),
        vec![update(world.carrier, "ordinary-carrier").push(create_intent(key))],
    )
}

/// C-15a: no succession. A raw lowercase prefix key sorts after a part's
/// scoped `WORTH.` key, and a prefix relation sorts after a stage's entity.
#[test]
fn coupled_creates_sorting_out_of_member_order_replay_with_parity() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let proposal = world
        .prepare_with_prefix(
            execution,
            create("zz-prefix"),
            &assemble(|execution, candidate| {
                ordinary_part(&world, execution, candidate, "a-companion")
            }),
        )
        .unwrap();
    publish_and_replay(&world, execution, proposal);
    let world = World::new(execution, true, true);
    let relation = link(
        "prefix-link",
        EntityReference::Existing(world.source),
        EntityReference::Existing(world.carrier),
    );
    let owners = world
        .runtime
        .readmit_edit_coupled_stage_owners(
            execution,
            &world.installed,
            &world.basis(),
            world.source,
            &[world.carrier],
        )
        .unwrap();
    let proposal = world
        .prepare_owners_with_prefix(
            execution,
            owners,
            WorkerIntentBatch::new("prefix-link").push(relation),
            &assemble(|execution, candidate| {
                candidate.append_authored_stage(
                    execution,
                    world.carrier,
                    &none(),
                    vec![update(world.carrier, "stage").push(create_intent("s"))],
                )
            }),
        )
        .unwrap();
    publish_and_replay(&world, execution, proposal);
}

/// The typed refusal a WAL replay of `world`'s history stops with.
fn replay_refusal(world: &World, execution: &ExecutionRequest<'_, '_>) -> String {
    let stop = world
        .replay(execution)
        .expect_err("an owned route's ownership completion is not replayed");
    let detail = format!("{stop:?}");
    let start = detail.find("denial: ").expect("a record allocation denial");
    detail[start..].split(" }").next().unwrap().to_owned()
}

/// C-15b: a #119 chain beside an authored stage that allocates between its
/// rounds; the chain's relation create sorts after the stage's entity. Tail
/// replay of an owned route's ownership completion is a separate durability
/// gap, #393 (its storage allocations are never replayed), so the oracle is the one
/// -round answer's replay: every plan allocation replays in canonical order
/// and both stop at the same ownership-completion residue.
#[test]
fn a_successor_chain_beside_an_allocating_stage_replays_its_plan_allocations() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let run = |values: &[&str]| {
        let world = World::new(execution, true, true);
        let round = |value: &str| {
            let mut batches = creates(&["x"], value);
            let x = EntityReference::Created(target("x"));
            batches[0] =
                batches[0]
                    .clone()
                    .push(link("x-link", x, EntityReference::Existing(world.source)));
            batches
        };
        let proposal = world
            .prepare_stages(
                execution,
                &[world.carrier],
                &assemble(|execution, candidate| {
                    candidate.append_publication(
                        execution,
                        birth(&world, execution, &["x"]),
                        &none(),
                        round(values[0]),
                    )?;
                    candidate.append_authored_stage(
                        execution,
                        world.carrier,
                        &none(),
                        vec![update(world.carrier, "stage").push(create_intent("s"))],
                    )?;
                    for value in &values[1..] {
                        let posture = birth(&world, execution, &["x"]);
                        successor(execution, candidate, posture, round(value))?;
                    }
                    Ok(())
                }),
            )
            .unwrap();
        let result = world.publish(execution, proposal);
        release_test_commit_snapshot(&world.runtime, &result);
        replay_refusal(&world, execution)
    };
    let expected = run(&["2"]);
    assert!(
        expected.starts_with("denial: ReplayEvidenceRemaining"),
        "{expected}"
    );
    assert_eq!(run(&["1", "2"]), expected);
}
