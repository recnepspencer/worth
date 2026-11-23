//! PR C C3a r1 note 3: the negative and replay cases the round courts left
//! out. A parent re-derived after its consumer's latest round is not
//! retained, a folded relation relink replays in a fresh process, and a
//! relation re-created with fields is a structural effect.
use super::rounds::linked_round;
use super::*;

fn second(world: &World, execution: &ExecutionRequest<'_, '_>) -> RelationalCoupledProducerPosture {
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
        admission: world.owner.admit_required_output("second", [5; 32]),
        absence: Some(world.absence(execution, "second")),
        meaning: world.meaning("second"),
        roles: vec![created("second-output")],
    })
}

/// The `owned` consumer (part 0) and the `second` parent (part 1), both owned
/// routes. The consumer re-runs after the parent and its latest round
/// retains it; then the parent derives again, after the consumer's latest
/// round, which can no longer retain it (`ForeignCoupledOwner`).
#[test]
fn a_parent_re_derived_after_its_consumers_latest_round_is_not_retained() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let mut world = World::new(execution, true, true);
    world.installed = world.installed.clone().with_source_scope_owned_producers([
        ("owned".into(), KindId(1)),
        ("second".into(), KindId(1)),
    ]);
    let answers = std::cell::RefCell::new(None);
    world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    birth(&world, execution, &["a"]),
                    &none(),
                    creates(&["a"], "1"),
                )?;
                candidate.append_publication(
                    execution,
                    second(&world, execution),
                    &none(),
                    vec![create("second-output")],
                )?;
                let inputs = |candidate: &Candidate<'_>| {
                    Ok::<_, TransactionCommitError>((
                        candidate
                            .applied_output_input(execution, &world.meaning("owned"))?
                            .expect("the consumer's input"),
                        candidate
                            .applied_output_input(execution, &world.meaning("second"))?
                            .expect("the parent's input"),
                    ))
                };
                successor(
                    execution,
                    candidate,
                    birth(&world, execution, &["a"]),
                    creates(&["a"], "1"),
                )?;
                let (consumer, parent) = inputs(candidate)?;
                let rerun = candidate
                    .retain_output_inputs(execution, &consumer, &[&parent])
                    .map_err(refusal);
                let token = candidate.next_producer_round(execution, 1)?;
                candidate.append_successor_publication(
                    execution,
                    token,
                    second(&world, execution),
                    &none(),
                    vec![create("second-output")],
                )?;
                let (consumer, parent) = inputs(candidate)?;
                let late = candidate
                    .retain_output_inputs(execution, &consumer, &[&parent])
                    .map_err(refusal);
                *answers.borrow_mut() = Some((rerun, late));
                Ok(())
            }),
        )
        .unwrap();
    assert_eq!(
        answers.into_inner(),
        Some((Ok(()), Err(Cause::ForeignCoupledOwner)))
    );
}

/// A Birth chain whose second round re-links its head's relation folds; a
/// fresh process reopened from its checkpoint reads the same records the
/// fold committed.
#[test]
fn a_folded_relation_relink_replays_in_a_fresh_process() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let keys = ["owned-output"];
    let meaning = world.meaning("owned");
    let main = PartitionId::main();
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    birth(&world, execution, &keys),
                    &none(),
                    linked_round(&world, &keys, "round-1", main, &keys),
                )?;
                successor(
                    execution,
                    candidate,
                    birth(&world, execution, &keys),
                    linked_round(&world, &keys, "round-2", main, &keys),
                )
            }),
        )
        .unwrap();
    let (observed, ids) = committed(&world, execution, proposal, &meaning);
    let (cold, _) = cold::reopen(&world, execution, true);
    assert_eq!(states(&cold::facts(&cold, execution, &ids)), observed.states);
}

/// Each round links its output to the source with a labelled relation: the
/// successor's relation carries fields, so it is a re-create the fold cannot
/// lower in place (`StructuralEffect`).
#[test]
fn a_relation_re_created_with_fields_is_a_structural_effect() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let keys = ["owned-output"];
    let main = PartitionId::main();
    let labelled = |value: &str| {
        let mut batches = linked_round(&world, &keys, value, main, &[]);
        let created = CreatedEntityRef {
            partition_id: main,
            kind_id: KindId(1),
            client_key: ClientKey::raw("owned-output"),
        };
        let batch = std::mem::replace(&mut batches[0], WorkerIntentBatch::new("round"));
        batches[0] = batch.push(MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: main,
            kind_id: KindId(2),
            client_key: ClientKey::raw("link:owned-output"),
            source: EntityReference::Created(created),
            target: EntityReference::Existing(world.source),
            fields: string_aspect_field_patch([(aspect_key("label"), field_key("label"), value)]),
        })));
        batches
    };
    let refused = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    birth(&world, execution, &keys),
                    &none(),
                    labelled("round-1"),
                )?;
                successor(
                    execution,
                    candidate,
                    birth(&world, execution, &keys),
                    labelled("round-2"),
                )
            }),
        )
        .unwrap_err();
    assert_eq!(refusal(refused), postponed(Reason::StructuralEffect));
}
