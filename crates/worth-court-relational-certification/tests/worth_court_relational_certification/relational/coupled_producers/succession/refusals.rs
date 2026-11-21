//! Negative succession courts: every non-folding round refuses with its
//! typed #119 reason or custody error without publishing anything.
use super::*;

#[test]
fn a_round_that_leaves_an_earlier_write_uncovered_refuses() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let before = world.basis();
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
                successor(execution, candidate, preserve(&world, &prior), Vec::new())
            }),
        )
        .unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::UncoveredWrite));
    // An allocation the head made and the round leaves out is uncovered too.
    let fresh = World::new(execution, true, true);
    let err = fresh
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                let keys = ["owned-output", "owned-second"];
                candidate.append_publication(
                    execution,
                    birth(&fresh, execution, &keys),
                    &none(),
                    creates(&keys, "round-1"),
                )?;
                successor(
                    execution,
                    candidate,
                    birth(&fresh, execution, &keys),
                    creates(&keys[..1], "round-2"),
                )
            }),
        )
        .unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::UncoveredWrite));
    assert_eq!(world.basis(), before, "nothing published");
}

#[test]
fn a_round_deleting_its_chains_allocation_refuses() {
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
    assert_eq!(refusal(err), postponed(Reason::FreshDelete));
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

#[test]
fn a_round_with_a_new_record_refuses_as_structural() {
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
                successor(
                    execution,
                    candidate,
                    birth(&world, execution, &["owned-output"]),
                    creates(&["owned-output", "owned-new"], "round-2"),
                )
            }),
        )
        .unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::StructuralEffect));
    assert_eq!(world.basis(), before);
    // A round writes each chain record once; two writes of one record are
    // not one round's answer.
    let prior = world.birth(execution, "owned", "owned-output");
    let before = world.basis();
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
                successor(
                    execution,
                    candidate,
                    preserve(&world, &prior),
                    vec![
                        update(output(&prior), "round-2"),
                        update(output(&prior), "round-3"),
                    ],
                )
            }),
        )
        .unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::StructuralEffect));
    assert_eq!(world.basis(), before);
}

#[test]
fn a_round_that_changes_its_roles_refuses() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let keys = ["owned-output", "owned-second"];
    let birth_with = |roles: Vec<RequiredOutputRoleTarget>| {
        let world = World::new(execution, true, true);
        let before = world.basis();
        let err = world
            .prepare(
                execution,
                &assemble(|execution, candidate| {
                    candidate.append_publication(
                        execution,
                        birth(&world, execution, &keys),
                        &none(),
                        creates(&keys, "round-1"),
                    )?;
                    successor(
                        execution,
                        candidate,
                        birth_roles(&world, execution, roles.clone()),
                        creates(&keys, "round-2"),
                    )
                }),
            )
            .unwrap_err();
        assert_eq!(world.basis(), before);
        refusal(err)
    };
    assert_eq!(
        birth_with(vec![created(keys[0])]),
        postponed(Reason::UncoveredWrite),
        "a dropped role"
    );
    let mut swapped = vec![created(keys[0]), created(keys[1])];
    swapped[0].role = keys[1].into();
    swapped[1].role = keys[0].into();
    assert_eq!(
        birth_with(swapped),
        postponed(Reason::UncoveredWrite),
        "swapped roles"
    );
    // A Preserve whose round retires its output is not the same chain.
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let before = world.basis();
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
                successor(
                    execution,
                    candidate,
                    preserve_as(&world, &prior, RequiredOutputRolePosture::Retire),
                    vec![update(output(&prior), "round-2")],
                )
            }),
        )
        .unwrap_err();
    assert_eq!(
        refusal(err),
        postponed(Reason::UncoveredWrite),
        "Preserve to Retire"
    );
    assert_eq!(world.basis(), before);
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
