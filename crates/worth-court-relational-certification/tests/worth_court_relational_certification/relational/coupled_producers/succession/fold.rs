//! Preserve, authored-stage composition, and two-width succession courts.
use super::*;

#[test]
fn a_preserve_round_folds_to_the_one_round_answer_and_reopens() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let write = |value: &str, output: Option<EntityId>| vec![update(output.unwrap(), value)];
    let run = |values: &[&str]| {
        let world = World::new(execution, true, true);
        let ((observed, ids), pin, committed) = chain_with_revisions(
            execution,
            &world,
            Route::Preserve,
            values,
            &write,
            &[("name", "name")],
        );
        (world, observed, ids, pin, committed)
    };
    let (world, actual, ids, pin_actual, committed_actual) = run(&["round-1", "round-2"]);
    let (_, expected, _, pin_expected, committed_expected) = run(&["round-2"]);
    assert_eq!(actual, expected);
    assert_eq!(pin_actual, pin_expected, "candidate pin revisions match");
    assert_eq!(
        committed_actual, committed_expected,
        "committed revisions match"
    );
    let (cold, _) = cold::reopen(&world, execution, true);
    assert_eq!(states(&cold::facts(&cold, execution, &ids)), actual.states);
    // C -> b -> C nets to what one round writing C publishes, including
    // aspect_versions and field_revisions on both the candidate pin and commit.
    let (_, back_actual, _, back_pin, back_committed) = run(&["round-1", "owned-output"]);
    let (_, back_expected, _, one_pin, one_committed) = run(&["owned-output"]);
    assert_eq!(back_actual, back_expected, "a chain back to the base value");
    assert_eq!(
        back_pin, one_pin,
        "C -> b -> C candidate pin keeps base aspect_versions and field_revisions"
    );
    assert_eq!(
        back_committed, one_committed,
        "C -> b -> C commit keeps base aspect_versions and field_revisions"
    );
}

#[test]
fn a_round_folds_beside_an_authored_stage_writer() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let run = |rounds: &[&str]| {
        let world = World::new(execution, true, true);
        let prior = world.birth(execution, "owned", "owned-output");
        let proposal = world
            .prepare_stages(
                execution,
                &[world.carrier],
                &assemble(|execution, candidate| {
                    candidate.append_publication(
                        execution,
                        preserve(&world, &prior),
                        &none(),
                        vec![update(output(&prior), rounds[0])],
                    )?;
                    candidate.append_authored_stage(
                        execution,
                        world.carrier,
                        &none(),
                        vec![update(world.carrier, "stage-carrier")],
                    )?;
                    for value in &rounds[1..] {
                        successor(
                            execution,
                            candidate,
                            preserve(&world, &prior),
                            vec![update(output(&prior), value)],
                        )?;
                    }
                    Ok(())
                }),
            )
            .unwrap();
        committed(&world, execution, proposal, &preserve_meaning(&prior)).0
    };
    assert_eq!(run(&["round-1", "round-2", "round-3"]), run(&["round-3"]));
}

/// Plan court 2: a stage composes on a disjoint field of the chain's output,
/// and a same-field stage write is still the ordinary `ConflictingIntent`.
#[test]
fn a_stage_beside_a_folded_chain_composes_on_disjoint_fields_only() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let run = |rounds: &[&str], field: &str| {
        let world = World::new_with_status(execution);
        let prior = world.birth(execution, "owned", "owned-output");
        let target = output(&prior);
        let stage = WorkerIntentBatch::new("stage").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: target,
                fields: string_aspect_field_patch([(
                    aspect_key(field),
                    field_key(field),
                    "staged",
                )]),
            }),
        ));
        let prepared = world.prepare_stages(
            execution,
            &[world.carrier],
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    preserve(&world, &prior),
                    &none(),
                    vec![update(target, rounds[0])],
                )?;
                for value in &rounds[1..] {
                    successor(
                        execution,
                        candidate,
                        preserve(&world, &prior),
                        vec![update(target, value)],
                    )?;
                }
                candidate.append_authored_stage(
                    execution,
                    world.carrier,
                    &none(),
                    vec![stage.clone()],
                )
            }),
        );
        prepared.map(|proposal| committed(&world, execution, proposal, &preserve_meaning(&prior)).0)
    };
    assert_eq!(
        run(&["round-1", "round-2"], "status").unwrap(),
        run(&["round-2"], "status").unwrap()
    );
    let error = run(&["round-1", "round-2"], "name").unwrap_err();
    assert!(
        matches!(error, TransactionCommitError::Conflict { ref error, .. }
        if matches!(error.class, ConflictClass::ConflictingIntent { .. })),
        "{error:?}"
    );
}

#[test]
fn a_folded_chain_costs_one_round_at_two_widths() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    for width in [4usize, 16] {
        let keys = (0..width)
            .map(|index| format!("owned-{index}"))
            .collect::<Vec<_>>();
        let keys = keys.iter().map(String::as_str).collect::<Vec<_>>();
        let examined = |rounds: &[&str]| {
            let world = World::new(execution, true, true);
            let proposal = world
                .prepare(
                    execution,
                    &assemble(|execution, candidate| {
                        candidate.append_publication(
                            execution,
                            birth(&world, execution, &keys),
                            &none(),
                            creates(&keys, rounds[0]),
                        )?;
                        for value in &rounds[1..] {
                            successor(
                                execution,
                                candidate,
                                birth(&world, execution, &keys),
                                creates(&keys, value),
                            )?;
                        }
                        Ok(())
                    }),
                )
                .unwrap();
            let work = proposal
                .mutation_touches(execution)
                .unwrap()
                .work()
                .validated_intents_examined();
            (
                work,
                committed(&world, execution, proposal, &world.meaning("owned")).0,
            )
        };
        assert_eq!(
            examined(&["round-1", "round-2", "round-3"]),
            examined(&["round-3"]),
            "width {width}: the folded chain validates and publishes one round"
        );
    }
}
