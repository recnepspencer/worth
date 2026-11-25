//! Succession courts formerly refused by the PR A fold: each round now
//! supersedes its chain's earlier round and the commit is the one-round
//! answer, or refuses with its typed #119 reason or ordinary cause without
//! publishing anything.
use super::*;

/// One Birth route in a fresh world: each entry is a round's roles and
/// creates; the first is appended, the rest are successors.
fn birth_rounds(rounds: &[(Vec<RequiredOutputRoleTarget>, Vec<WorkerIntentBatch>)]) -> Observed {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                let (roles, batches) = &rounds[0];
                candidate.append_publication(
                    execution,
                    birth_roles(&world, execution, roles.clone()),
                    &none(),
                    batches.clone(),
                )?;
                for (roles, batches) in &rounds[1..] {
                    let posture = birth_roles(&world, execution, roles.clone());
                    successor(execution, candidate, posture, batches.clone())?;
                }
                Ok(())
            }),
        )
        .unwrap();
    committed(&world, execution, proposal, &world.meaning("owned")).0
}

/// One Preserve route in a fresh world: each entry is a round's role posture
/// and output value (`None` writes nothing).
fn preserve_rounds(rounds: &[(RequiredOutputRolePosture, Option<&str>)]) -> Observed {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let round = |value: Option<&str>| {
        value
            .map(|value| update(output(&prior), value))
            .into_iter()
            .collect()
    };
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                let (posture, value) = rounds[0];
                candidate.append_publication(
                    execution,
                    preserve_as(&world, &prior, posture),
                    &none(),
                    round(value),
                )?;
                for &(posture, value) in &rounds[1..] {
                    successor(
                        execution,
                        candidate,
                        preserve_as(&world, &prior, posture),
                        round(value),
                    )?;
                }
                Ok(())
            }),
        )
        .unwrap();
    committed(&world, execution, proposal, &preserve_meaning(&prior)).0
}

/// The refusal one Preserve route's rounds stop with, having published nothing.
fn preserve_refusal(rounds: &[(RequiredOutputRolePosture, Option<&str>)]) -> Cause {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let before = world.basis();
    let round = |value: Option<&str>| {
        value
            .map(|value| update(output(&prior), value))
            .into_iter()
            .collect()
    };
    let err = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                let (posture, value) = rounds[0];
                candidate.append_publication(
                    execution,
                    preserve_as(&world, &prior, posture),
                    &none(),
                    round(value),
                )?;
                for &(posture, value) in &rounds[1..] {
                    successor(
                        execution,
                        candidate,
                        preserve_as(&world, &prior, posture),
                        round(value),
                    )?;
                }
                Ok(())
            }),
        )
        .and_then(|proposal| {
            world
                .runtime
                .prepare_validated_proposal(execution, proposal)
                .map(|_| ())
        })
        .unwrap_err();
    assert_eq!(world.basis(), before);
    refusal(err)
}

fn roles_of(keys: &[&str]) -> Vec<RequiredOutputRoleTarget> {
    keys.iter().map(|key| created(key)).collect()
}

/// A round that no longer writes what an earlier round wrote reverts it to
/// the head's image (N5); one that no longer creates an earlier record drops
/// it (N2). Both commit the one-round answer.
#[test]
fn a_round_leaving_out_an_earlier_write_or_create_commits_the_one_round_answer() {
    use RequiredOutputRolePosture::Preserve as Keep;
    assert_eq!(
        preserve_rounds(&[(Keep, Some("round-1")), (Keep, None)]),
        preserve_rounds(&[(Keep, None)]),
        "an uncovered field write reverts"
    );
    let keys = ["owned-output", "owned-second"];
    assert_eq!(
        birth_rounds(&[
            (roles_of(&keys), creates(&keys, "round-1")),
            (roles_of(&keys[..1]), creates(&keys[..1], "round-2")),
        ])
        .alpha(),
        birth_rounds(&[(roles_of(&keys[..1]), creates(&keys[..1], "round-2"))]).alpha(),
        "an omitted create is dropped"
    );
}

/// Naming a record only a superseded round of the chain created can only
/// come from reading that round's effect: `PriorEffectRead`.
#[test]
fn a_round_naming_its_chains_superseded_allocation_refuses_prior_effect_read() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let before = world.basis();
    let err = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    birth(&world, execution, &["owned-output"]),
                    &none(),
                    creates(&["owned-output"], "round-1"),
                )?;
                let fresh = candidate
                    .appended_output_entities(execution, "owned", &world.meaning("owned"))?
                    .expect("the appended route")[0];
                successor(
                    execution,
                    candidate,
                    birth(&world, execution, &["owned-output"]),
                    vec![
                        WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
                            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: fresh }),
                        )),
                    ],
                )
            }),
        )
        .unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::PriorEffectRead));
    assert_eq!(world.basis(), before);
}

#[test]
fn a_round_after_another_writer_changed_its_output_refuses() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
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
                    vec![update(world.carrier, "round-1")],
                )?;
                candidate.append_authored_stage(
                    execution,
                    world.carrier,
                    &none(),
                    vec![update(world.carrier, "stage-carrier")],
                )?;
                successor(
                    execution,
                    candidate,
                    preserve(&world, &prior),
                    vec![update(world.carrier, "round-2")],
                )
            }),
        )
        .unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::IntervenedOutput));
    assert_eq!(world.basis(), before);
}

/// A round creating a record its head did not is an ordinary create; the
/// commit equals the one-round answer (exact: no other key was added after).
#[test]
fn a_round_with_a_new_record_commits_the_one_round_answer() {
    let keys = ["owned-output", "owned-new"];
    assert_eq!(
        birth_rounds(&[
            (roles_of(&keys[..1]), creates(&keys[..1], "round-1")),
            (roles_of(&keys[..1]), creates(&keys, "round-2")),
        ]),
        birth_rounds(&[(roles_of(&keys[..1]), creates(&keys, "round-2"))]),
    );
}

/// A round is applied as written: two writes of one record in a round are
/// that round's own ordinary conflict.
#[test]
fn a_round_writing_one_record_twice_is_the_ordinary_conflict() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let before = world.basis();
    let twice = || {
        vec![
            update(output(&prior), "round-2"),
            update(output(&prior), "round-3"),
        ]
    };
    let err = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    preserve(&world, &prior),
                    &none(),
                    vec![update(output(&prior), "round-1")],
                )?;
                successor(execution, candidate, preserve(&world, &prior), twice())
            }),
        )
        .unwrap_err();
    assert!(
        matches!(err, TransactionCommitError::Conflict { .. }),
        "{err:?}"
    );
    assert_eq!(world.basis(), before);
}

/// A round may change its roles: drop one, swap two, or retire a preserved
/// output. Its posture and ordinary roles are re-resolved; the commit is the
/// one-round answer.
#[test]
fn a_round_that_changes_its_roles_commits_the_one_round_answer() {
    let keys = ["owned-output", "owned-second"];
    let head = || (roles_of(&keys), creates(&keys, "round-1"));
    let mut swapped = roles_of(&keys);
    swapped[0].role = keys[1].into();
    swapped[1].role = keys[0].into();
    for roles in [roles_of(&keys[..1]), swapped] {
        assert_eq!(
            birth_rounds(&[head(), (roles.clone(), creates(&keys, "round-2"))]),
            birth_rounds(&[(roles, creates(&keys, "round-2"))]),
        );
    }
    // Retiring a preserved output is refused for one round alone; after an
    // earlier Preserve round it is the same ordinary refusal.
    use RequiredOutputRolePosture::{Preserve as Keep, Retire};
    assert_eq!(
        preserve_refusal(&[(Keep, Some("round-1")), (Retire, Some("round-2"))]),
        preserve_refusal(&[(Retire, Some("round-2"))]),
        "Preserve to Retire"
    );
}

#[test]
fn stale_tokens_other_postures_and_second_entries_refuse() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let head = |candidate: &mut Candidate<'_>, execution: &ExecutionRequest<'_, '_>| {
        candidate.append_publication(
            execution,
            preserve(&world, &prior),
            &none(),
            vec![update(output(&prior), "round-1")],
        )
    };
    let round = || vec![update(output(&prior), "round-2")];
    let stale = assemble(|execution, candidate| {
        head(candidate, execution)?;
        let token = candidate.next_producer_round(execution, 0)?;
        let _newer = candidate.next_producer_round(execution, 0)?;
        candidate.append_successor_publication(
            execution,
            token,
            preserve(&world, &prior),
            &none(),
            round(),
        )
    });
    let other_kind = assemble(|execution, candidate| {
        head(candidate, execution)?;
        // The posture kind is judged before its absence is.
        let other =
            RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
                admission: world.owner.admit_required_output("owned", [3; 32]),
                absence: None,
                meaning: world.meaning("owned"),
                roles: vec![created("owned-output")],
            });
        successor(execution, candidate, other, round())
    });
    let second_entry = assemble(|execution, candidate| {
        head(candidate, execution)?;
        successor(execution, candidate, preserve(&world, &prior), round())?;
        candidate.append_publication(execution, preserve(&world, &prior), &none(), round())
    });
    for (case, assembly, cause) in [
        (
            "stale",
            &stale as &dyn RelationalAuthoredCoupledProducerAssembly,
            Cause::StaleCoupledContinuation,
        ),
        (
            "other posture kind",
            &other_kind,
            Cause::RepeatedCoupledOwner,
        ),
        (
            "second first entry",
            &second_entry,
            Cause::RepeatedCoupledOwner,
        ),
    ] {
        let before = world.basis();
        let err = world.prepare(execution, assembly).unwrap_err();
        assert_eq!(refusal(err), cause, "{case}");
        assert_eq!(world.basis(), before, "{case}");
    }
}
