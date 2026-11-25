//! C-13: the CAD body shape (#119 §1.3). The body route's head creates its
//! cells; an authored stage edits after it; the next round creates a cell
//! under a new key, retires a cell, links a cell to the stage's record,
//! changes its roles and carries content. It commits the one-round answer.
use super::*;
use worth_foundational::facade::AspectContract;

/// One body round: cells `keys`, the first carrying content `first`, and
/// with `link` a relation from the last cell to the carrier.
fn body_round(
    world: &World,
    contract: &AspectContract,
    execution: &ExecutionRequest<'_, '_>,
    candidate: &mut Candidate<'_>,
    keys: &[&str],
    first: i64,
    link_carrier: bool,
) -> Result<(), Failure> {
    let frame = world.frame(execution, first);
    let mut batch = WorkerIntentBatch::new("body").push(content_create(contract, keys[0], &frame));
    for key in &keys[1..] {
        batch = batch.push(create_intent(key));
    }
    if link_carrier {
        let last = EntityReference::Created(target(keys[keys.len() - 1]));
        batch = batch.push(link(
            "body-link",
            last,
            EntityReference::Existing(world.carrier),
        ));
    }
    let content = attachments(execution, &[&frame]);
    let posture = birth(world, execution, keys);
    if candidate.continues_owned_route(0) {
        let token = candidate.next_producer_round(execution, 0)?;
        candidate.append_successor_publication(execution, token, posture, &content, vec![batch])
    } else {
        candidate.append_publication(execution, posture, &content, vec![batch])
    }
}

/// The first stage edits the carrier; a later one creates its own record.
fn stage_batch(world: &World, index: usize) -> WorkerIntentBatch {
    if index == 0 {
        update(world.carrier, "stage-0")
    } else {
        create(&format!("stage-{index}"))
    }
}

/// The shape's states after `rounds` body rounds around `stages` stages.
fn shape(stages: usize, rounds: &[(&[&str], i64, bool)]) -> (Vec<String>, Vec<String>) {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let (world, contract) = World::new_with_content(execution);
    let later = vec![world.carrier; stages];
    let proposal = world
        .prepare_stages(
            execution,
            &later,
            &assemble(|execution, candidate| {
                for (index, &(keys, first, linked)) in rounds.iter().enumerate() {
                    body_round(&world, &contract, execution, candidate, keys, first, linked)?;
                    if index < stages {
                        let stage = stage_batch(&world, index);
                        candidate.append_authored_stage(
                            execution,
                            world.carrier,
                            &none(),
                            vec![stage],
                        )?;
                    }
                }
                for index in rounds.len()..stages {
                    let stage = stage_batch(&world, index);
                    candidate.append_authored_stage(
                        execution,
                        world.carrier,
                        &none(),
                        vec![stage],
                    )?;
                }
                Ok(())
            }),
        )
        .unwrap();
    let result = world.publish(execution, proposal);
    let mut roles = Vec::new();
    let outputs = result
        .required_output_publications(&world.installed, execution)
        .flat_map(|origin| {
            origin
                .unwrap()
                .members()
                .map(|member| {
                    roles.push(member.0.to_owned());
                    member.3
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    release_test_commit_snapshot(&world.runtime, &result);
    let mut ids = vec![world.source, world.carrier];
    ids.extend(outputs);
    (states(&cold::facts(&world.runtime, execution, &ids)), roles)
}

#[test]
fn the_cad_body_shape_commits_the_one_round_answer() {
    let head: (&[&str], i64, bool) = (&["cell-1", "cell-2"], 1, false);
    let next: (&[&str], i64, bool) = (&["cell-2", "cell-3"], 2, true);
    assert_eq!(shape(1, &[head, next]), shape(1, &[next]), "two stages");
    let last: (&[&str], i64, bool) = (&["cell-4", "cell-3"], 3, true);
    assert_eq!(
        shape(2, &[head, next, last]),
        shape(2, &[last]),
        "three rounds, two stages"
    );
}
