//! C-17: several chains interleaved (#119 §3.1, Neon 4b; Lean T4m).
use super::*;

/// The world with a second owned producer family, `second`.
fn two_routes(mut world: World) -> World {
    world.installed = world.installed.clone().with_source_scope_owned_producers([
        ("owned".into(), KindId(1)),
        ("second".into(), KindId(1)),
    ]);
    world
}

fn posture(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    family: &str,
    keys: &[&str],
) -> RelationalCoupledProducerPosture {
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
        admission: world.owner.admit_required_output(family, [3; 32]),
        absence: Some(world.absence(execution, family)),
        meaning: world.meaning(family),
        roles: keys.iter().map(|key| created(key)).collect(),
    })
}

/// One round of chain `index` (0: `owned`, 1: `second`) creating `keys`,
/// plus `extra` batches.
fn round(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    candidate: &mut Candidate<'_>,
    index: usize,
    keys: &[&str],
    extra: Vec<WorkerIntentBatch>,
) -> Result<(), Failure> {
    let family = ["owned", "second"][index];
    let mut batches = creates(keys, family);
    batches.extend(extra);
    if candidate.continues_owned_route(index) {
        let token = candidate.next_producer_round(execution, index)?;
        candidate.append_successor_publication(
            execution,
            token,
            posture(world, execution, family, keys),
            &none(),
            batches,
        )
    } else {
        candidate.append_publication(
            execution,
            posture(world, execution, family, keys),
            &none(),
            batches,
        )
    }
}

fn outputs_of(
    candidate: &mut Candidate<'_>,
    execution: &ExecutionRequest<'_, '_>,
    world: &World,
    family: &str,
) -> Result<Vec<EntityId>, Failure> {
    Ok(candidate
        .appended_output_entities(execution, family, &world.meaning(family))?
        .expect("appended")
        .to_vec())
}

/// (i) A chain that dropped its link to another chain's record before that
/// record is cancelled causes no false `RetiredReference`.
#[test]
fn a_dropped_link_to_a_cancelled_record_is_no_retired_reference() {
    run_in(two_routes, false, &|world, execution, candidate| {
        round(world, execution, candidate, 0, &["x1", "x2"], Vec::new())?;
        let x1 = outputs_of(candidate, execution, world, "owned")?[0];
        let to_x1 = WorkerIntentBatch::new("link").push(link(
            "y-link",
            EntityReference::Created(target("y")),
            EntityReference::Existing(x1),
        ));
        round(world, execution, candidate, 1, &["y"], vec![to_x1])?;
        round(world, execution, candidate, 1, &["y"], Vec::new())?;
        round(world, execution, candidate, 0, &["x2"], Vec::new())
    })
    .expect("no live member names the cancelled record");
}

/// (ii) A chain that reverted its write of a record before another chain's
/// round deletes it causes no false `IntervenedOutput`.
#[test]
fn a_reverted_write_is_no_intervention() {
    run_in(two_routes, false, &|world, execution, candidate| {
        round(world, execution, candidate, 0, &["x"], Vec::new())?;
        round(
            world,
            execution,
            candidate,
            1,
            &["y"],
            vec![update(world.carrier, "second")],
        )?;
        round(world, execution, candidate, 1, &["y"], Vec::new())?;
        round(
            world,
            execution,
            candidate,
            0,
            &["x"],
            vec![update(world.carrier, "owned-2")],
        )
    })
    .expect("the reverted write is no live member's");
}

/// (iii) The first chain's round 2 may rewrite a record the second chain
/// wrote and then reverted; the second chain's next round is then blocked
/// for good, since its touched set only grows.
#[test]
fn a_later_write_of_a_touched_record_intervenes_for_good() {
    let rounds = |third: bool| {
        run_in(two_routes, false, &|world, execution, candidate| {
            round(
                world,
                execution,
                candidate,
                0,
                &["x"],
                vec![update(world.carrier, "owned-1")],
            )?;
            round(
                world,
                execution,
                candidate,
                1,
                &["y"],
                vec![update(world.carrier, "second-1")],
            )?;
            round(world, execution, candidate, 1, &["y"], Vec::new())?;
            round(
                world,
                execution,
                candidate,
                0,
                &["x"],
                vec![update(world.carrier, "owned-2")],
            )?;
            if third {
                round(world, execution, candidate, 1, &["y"], Vec::new())?;
            }
            Ok(())
        })
    };
    rounds(false).expect("the reverted write no longer intervenes");
    assert_eq!(
        refusal(rounds(true).err().expect("blocked")),
        postponed(Reason::IntervenedOutput)
    );
}

/// (iv) Interleaved chains commit what each chain's last round alone would
/// at its head's position.
#[test]
fn interleaved_chains_commit_the_reference_answer() {
    let chains = run_in(two_routes, false, &|world, execution, candidate| {
        round(
            world,
            execution,
            candidate,
            0,
            &["x", "dropped"],
            Vec::new(),
        )?;
        round(world, execution, candidate, 1, &["y"], Vec::new())?;
        round(world, execution, candidate, 0, &["x"], Vec::new())?;
        round(world, execution, candidate, 1, &["y", "z"], Vec::new())
    })
    .unwrap();
    let reference = run_in(two_routes, false, &|world, execution, candidate| {
        round(world, execution, candidate, 0, &["x"], Vec::new())?;
        round(world, execution, candidate, 1, &["y", "z"], Vec::new())
    })
    .unwrap();
    assert_eq!(chains.observed, reference.observed);
    assert_eq!(
        chains.outputs.keys().collect::<Vec<_>>(),
        reference.outputs.keys().collect::<Vec<_>>()
    );
}

/// (v) Lean `MultiChainGap`: an earlier-headed chain that re-rounded after
/// this chain's head may have retracted a record past the image this chain's
/// head saw; this chain may not then change it (stale-base guard).
#[test]
fn a_round_changing_a_record_an_earlier_chain_retracted_is_intervened() {
    let cause = refusal(
        run_in(two_routes, false, &|world, execution, candidate| {
            round(
                world,
                execution,
                candidate,
                0,
                &["x"],
                vec![update(world.carrier, "owned-1")],
            )?;
            round(world, execution, candidate, 1, &["y"], Vec::new())?;
            round(world, execution, candidate, 0, &["x"], Vec::new())?;
            round(
                world,
                execution,
                candidate,
                1,
                &["y"],
                vec![update(world.carrier, "second-2")],
            )
        })
        .err()
        .expect("the stale-base guard refuses"),
    );
    assert_eq!(cause, postponed(Reason::IntervenedOutput));
}
