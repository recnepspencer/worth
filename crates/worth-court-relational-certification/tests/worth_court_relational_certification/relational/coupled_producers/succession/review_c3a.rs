//! C3a review courts (r1 blockers 4 and 5): a chain's exclusive records are
//! judged over everything written since its head, and a round re-creates its
//! head's relations one to one by key and partition.
use super::*;

/// One Birth round of `owned-output`, linked (fieldless) by each relation
/// `(key, target)` from the output to `target`.
fn relation_round(value: &str, relations: &[(&str, EntityId)]) -> Vec<WorkerIntentBatch> {
    let output = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw("owned-output"),
    };
    let batch = relations.iter().fold(
        WorkerIntentBatch::new("round").push(named("owned-output", value)),
        |batch, (key, target)| {
            batch.push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw(*key),
                    source: EntityReference::Created(output.clone()),
                    target: EntityReference::Existing(*target),
                    fields: AspectFieldPatch::default(),
                },
            )))
        },
    );
    vec![batch]
}

/// A Birth chain of `rounds`, each `(value, relations)`; the commit's
/// observation, or the round's refusal.
fn relation_chain(
    rounds: &[(&str, Vec<(&str, fn(&World) -> EntityId)>)],
) -> Result<Observed, Cause> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let meaning = world.meaning("owned");
    let round = |(value, relations): &(&str, Vec<(&str, fn(&World) -> EntityId)>)| {
        let relations = relations
            .iter()
            .map(|(key, target)| (*key, target(&world)))
            .collect::<Vec<_>>();
        relation_round(value, &relations)
    };
    let prepared = world.prepare(
        execution,
        &assemble(|execution, candidate| {
            candidate.append_publication(
                execution,
                birth(&world, execution, &["owned-output"]),
                &none(),
                round(&rounds[0]),
            )?;
            for later in &rounds[1..] {
                successor(
                    execution,
                    candidate,
                    birth(&world, execution, &["owned-output"]),
                    round(later),
                )?;
            }
            Ok(())
        }),
    );
    match prepared {
        Ok(proposal) => Ok(committed(&world, execution, proposal, &meaning).0),
        Err(err) => Err(refusal(err)),
    }
}

fn source(world: &World) -> EntityId {
    world.source
}
fn carrier(world: &World) -> EntityId {
    world.carrier
}

/// Two relations of one identity (partition, kind, source, target) cannot
/// exist in one candidate: Native refuses the round that creates them, so a
/// chain never holds parallel relations a fold could confuse.
#[test]
fn parallel_relations_of_one_identity_are_refused_before_any_fold() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let err = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    birth(&world, execution, &["owned-output"]),
                    &none(),
                    relation_round(
                        "round-1",
                        &[("link:a", world.source), ("link:b", world.source)],
                    ),
                )
            }),
        )
        .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = err else {
        panic!("a duplicate relation identity conflict: {err:?}")
    };
    assert_eq!(
        error.code,
        worth_relational::facade::diagnostics::DiagnosticCode::DuplicateRelationIdentity
    );
}

/// Dropping one of two head relations drops it: never published, its
/// identity cancelled (#119 N2). The commit is the one-round answer.
#[test]
fn a_round_dropping_a_head_relation_commits_the_one_round_answer() {
    let round_2 = || {
        (
            "round-2",
            vec![("link:a", source as fn(&World) -> EntityId)],
        )
    };
    let rounds = relation_chain(&[
        (
            "round-1",
            vec![
                ("link:a", source as fn(&World) -> EntityId),
                ("link:b", carrier),
            ],
        ),
        round_2(),
    ]);
    assert_eq!(
        rounds.unwrap().alpha(),
        relation_chain(&[round_2()]).unwrap().alpha()
    );
}

/// A round that swaps its head relations' keys across their endpoints
/// re-creates each key on its new endpoints; each key keeps its own identity
/// (#119 M-ID). The commit is the one-round answer.
#[test]
fn a_round_swapping_its_head_relation_keys_commits_the_one_round_answer() {
    let round_2 = || {
        (
            "round-2",
            vec![
                ("link:a", carrier as fn(&World) -> EntityId),
                ("link:b", source),
            ],
        )
    };
    let rounds = relation_chain(&[
        (
            "round-1",
            vec![
                ("link:a", source as fn(&World) -> EntityId),
                ("link:b", carrier),
            ],
        ),
        round_2(),
    ]);
    assert_eq!(
        rounds.unwrap().alpha(),
        relation_chain(&[round_2()]).unwrap().alpha()
    );
}

fn status(entity: EntityId, value: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("status-write").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: entity,
            fields: string_aspect_field_patch([(aspect_key("status"), field_key("status"), value)]),
        }),
    ))
}

/// A stage writes the carrier after the chain's head; round 2 then joins the
/// carrier to the chain. Round 3's own-output view would read the carrier as
/// it was before the head, hiding the stage's write, so it refuses.
#[test]
fn a_record_written_since_the_head_and_joined_later_is_intervened() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new_with_status(execution);
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
                    vec![update(output(&prior), "round-1")],
                )?;
                candidate.append_authored_stage(
                    execution,
                    world.carrier,
                    &none(),
                    vec![status(world.carrier, "stage")],
                )?;
                successor(
                    execution,
                    candidate,
                    preserve(&world, &prior),
                    vec![
                        update(output(&prior), "round-2"),
                        update(world.carrier, "round-2"),
                    ],
                )?;
                successor(
                    execution,
                    candidate,
                    preserve(&world, &prior),
                    vec![
                        update(output(&prior), "round-3"),
                        update(world.carrier, "round-3"),
                    ],
                )
            }),
        )
        .unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::IntervenedOutput));
    assert_eq!(world.basis(), before);
}
