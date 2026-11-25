//! C-11: content attachments of superseded rounds and of every live member
//! (#119 D6). The commit's content is the live members' union, each content
//! record once, prepared against the basis.
use super::*;
use worth_foundational::facade::AspectContract;

/// Publish an assembly in a fresh content world; `prefix` attaches its own
/// frame to the authored prefix. Returns the committed states of the route's
/// outputs, or the refusal.
fn content_run(
    stage: bool,
    prefix: bool,
    assembly: &dyn Fn(
        &World,
        &AspectContract,
        &ExecutionRequest<'_, '_>,
        &mut Candidate<'_>,
    ) -> Result<(), Failure>,
) -> Result<Vec<String>, TransactionCommitError> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let (world, contract) = World::new_with_content(execution);
    let assembled =
        assemble(|execution, candidate| assembly(&world, &contract, execution, candidate));
    let proposal = if prefix {
        let frame = world.frame(execution, 7);
        let batch =
            WorkerIntentBatch::new("prefix").push(content_create(&contract, "prefix-grid", &frame));
        world.prepare_with_content_prefix(
            execution,
            batch,
            &attachments(execution, &[&frame]),
            &assembled,
        )?
    } else if stage {
        world.prepare_stages(execution, &[world.carrier], &assembled)?
    } else {
        world.prepare(execution, &assembled)?
    };
    let candidate = world
        .runtime
        .prepare_validated_proposal(execution, proposal)?;
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the complete group performs")
    };
    let result = world.runtime.settle_performed_publication(performed)?;
    let outputs = result
        .required_output_publications(&world.installed, execution)
        .flat_map(|origin| {
            origin
                .unwrap()
                .members()
                .map(|member| member.3)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    release_test_commit_snapshot(&world.runtime, &result);
    Ok(states(&cold::facts(&world.runtime, execution, &outputs)))
}

/// One content round of the owned route: create `key` over a frame of `first`.
fn content_round(
    world: &World,
    contract: &AspectContract,
    execution: &ExecutionRequest<'_, '_>,
    candidate: &mut Candidate<'_>,
    key: &str,
    first: i64,
) -> Result<(), Failure> {
    let frame = world.frame(execution, first);
    let batches = vec![WorkerIntentBatch::new(key).push(content_create(contract, key, &frame))];
    let content = attachments(execution, &[&frame]);
    let posture = birth(world, execution, &[key]);
    if candidate.continues_owned_route(0) {
        let token = candidate.next_producer_round(execution, 0)?;
        candidate.append_successor_publication(execution, token, posture, &content, batches)
    } else {
        candidate.append_publication(execution, posture, &content, batches)
    }
}

/// (a) A round superseding a content round drops its record and its
/// attachment: no `UnreferencedAttachment`, and the one-round answer.
#[test]
fn a_superseded_rounds_content_is_never_published() {
    let chain = content_run(false, false, &|world, contract, execution, candidate| {
        content_round(world, contract, execution, candidate, "x", 1)?;
        content_round(world, contract, execution, candidate, "y", 2)
    })
    .unwrap();
    let one = content_run(false, false, &|world, contract, execution, candidate| {
        content_round(world, contract, execution, candidate, "y", 2)
    })
    .unwrap();
    assert_eq!(chain, one);
}

/// (b) A stage attaching the same content the chain drops keeps it, once.
#[test]
fn content_shared_with_a_live_stage_stays_once() {
    content_run(true, false, &|world, contract, execution, candidate| {
        content_round(world, contract, execution, candidate, "x", 1)?;
        let frame = world.frame(execution, 1);
        let stage =
            update(world.carrier, "stage").push(content_create(contract, "stage-grid", &frame));
        candidate.append_authored_stage(
            execution,
            world.carrier,
            &attachments(execution, &[&frame]),
            vec![stage],
        )?;
        content_round(world, contract, execution, candidate, "y", 2)
    })
    .unwrap();
}

/// (c) A prefix's content survives a later member that attaches none: the
/// commit's content is every live member's, not the last suffix's.
#[test]
fn a_prefix_content_survives_a_later_contentless_member() {
    content_run(false, true, &|world, _, execution, candidate| {
        candidate.append_publication(
            execution,
            birth(world, execution, &["plain"]),
            &none(),
            creates(&["plain"], "1"),
        )
    })
    .unwrap();
}
