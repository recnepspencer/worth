//! PR C successor rounds through real Query-shaped posture (C3): a round
//! re-links its head's relation, a round issued in another partition names
//! the head's records, a reopened Preserve route reads the owner its part
//! claimed, and a re-run consumer retains a parent appended after its head.
use super::*;
use std::cell::Cell;

/// One Birth round of `keys` in `partition`, each output linked (fieldless) to
/// the world's source by `links`, keyed `link:<key>`.
fn linked_round(
    world: &World,
    keys: &[&str],
    value: &str,
    partition: PartitionId,
    links: &[&str],
) -> Vec<WorkerIntentBatch> {
    let created = |key: &str| CreatedEntityRef {
        partition_id: partition,
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
    };
    let entities = keys.iter().fold(WorkerIntentBatch::new("round"), |batch, key| {
        batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: partition,
            kind_id: KindId(1),
            client_key: ClientKey::raw(*key),
            fields: string_aspect_field_patch([(
                aspect_key("name"),
                field_key("name"),
                format!("{key}:{value}").as_str(),
            )]),
        })))
    });
    let batch = links.iter().fold(entities, |batch, key| {
        batch.push(MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: partition,
            kind_id: KindId(2),
            client_key: ClientKey::raw(format!("link:{key}")),
            source: EntityReference::Created(created(key)),
            target: EntityReference::Existing(world.source),
            fields: AspectFieldPatch::default(),
        })))
    });
    vec![batch]
}

/// A Birth chain whose rounds are `rounds` (value, partition, links), folded.
fn linked_chain(
    rounds: &[(&str, PartitionId, &[&str])],
) -> Result<(Observed, Vec<EntityId>), Cause> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let keys = ["owned-output"];
    let meaning = world.meaning("owned");
    let prepared = world.prepare(
        execution,
        &assemble(|execution, candidate| {
            let (value, partition, links) = rounds[0];
            candidate.append_publication(
                execution,
                birth(&world, execution, &keys),
                &none(),
                linked_round(&world, &keys, value, partition, links),
            )?;
            for &(value, partition, links) in &rounds[1..] {
                successor(
                    execution,
                    candidate,
                    birth(&world, execution, &keys),
                    linked_round(&world, &keys, value, partition, links),
                )?;
            }
            Ok(())
        }),
    );
    match prepared {
        Ok(proposal) => Ok(committed(&world, execution, proposal, &meaning)),
        Err(err) => Err(refusal(err)),
    }
}

/// A Birth round that re-creates its head's relation (same kind, same
/// endpoints, fieldless) folds: the commit equals the one-round answer, and
/// the chain still creates the relation once.
#[test]
fn a_birth_round_relinking_its_head_relation_folds() {
    let main = PartitionId::main();
    let folded = linked_chain(&[
        ("round-1", main, &["owned-output"]),
        ("round-2", main, &["owned-output"]),
    ])
    .unwrap();
    let one = linked_chain(&[("round-2", main, &["owned-output"])]).unwrap();
    assert_eq!(folded.0, one.0);
}

/// A relation the head never created is a new record the fold cannot lower.
#[test]
fn a_successor_round_adding_a_relation_is_a_structural_effect() {
    let main = PartitionId::main();
    let refused = linked_chain(&[("round-1", main, &[]), ("round-2", main, &["owned-output"])]);
    assert_eq!(refused.unwrap_err(), postponed(Reason::StructuralEffect));
}

/// A successor that does not re-create its head's relation leaves it uncovered.
#[test]
fn a_successor_round_dropping_its_head_relation_is_uncovered() {
    let main = PartitionId::main();
    let refused = linked_chain(&[("round-1", main, &["owned-output"]), ("round-2", main, &[])]);
    assert_eq!(refused.unwrap_err(), postponed(Reason::UncoveredWrite));
}

/// Each Query attempt issues its own mutation partition. A successor whose
/// creates and created references name another partition still names its
/// chain's records by key: it folds onto the head's allocation, equal to the
/// one-round answer in the head's partition.
#[test]
fn a_successor_round_in_another_partition_folds_onto_the_heads_records() {
    let main = PartitionId::main();
    let other = PartitionId(7);
    let folded = linked_chain(&[
        ("round-1", main, &["owned-output"]),
        ("round-2", other, &["owned-output"]),
    ])
    .unwrap();
    let one = linked_chain(&[("round-2", main, &["owned-output"])]).unwrap();
    assert_eq!(folded.0, one.0);
}

/// A reopened Preserve route reads the owner its existing part claimed:
/// `read_parent_owner` refuses the seen route, `read_successor_parent_owner`
/// returns the same surviving owner it returned before the part appended.
#[test]
fn a_successor_reads_the_parent_owner_its_part_claimed() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let meaning = preserve_meaning(&prior);
    let seen = Cell::new(None);
    world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                let surviving = |owner: Option<RelationalDeclaredCoupledOwner<'_>>| {
                    matches!(owner, Some(RelationalDeclaredCoupledOwner::Surviving(_)))
                };
                let partition = *meaning.source_partition.bytes();
                let before = surviving(candidate.read_successor_parent_owner(
                    execution,
                    "owned",
                    meaning.source_scope,
                    &partition,
                )?);
                candidate.append_publication(
                    execution,
                    preserve(&world, &prior),
                    &none(),
                    vec![update(output(&prior), "round-1")],
                )?;
                let ordinary = candidate
                    .read_parent_owner(execution, "owned", meaning.source_scope, &partition)
                    .map(|_| ())
                    .map_err(refusal);
                let successor = surviving(candidate.read_successor_parent_owner(
                    execution,
                    "owned",
                    meaning.source_scope,
                    &partition,
                )?);
                seen.set(Some((before, ordinary, successor)));
                Ok(())
            }),
        )
        .unwrap();
    assert_eq!(
        seen.take(),
        Some((true, Err(Cause::RepeatedCoupledOwner), true))
    );
}

fn second(world: &World, execution: &ExecutionRequest<'_, '_>) -> RelationalCoupledProducerPosture {
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
        admission: world.owner.admit_required_output("second", [5; 32]),
        absence: Some(world.absence(execution, "second")),
        meaning: world.meaning("second"),
        roles: vec![created("second-output")],
    })
}

/// The `owned` consumer appends first (part 0), then the `second` parent
/// (part 1). The consumer's head round cannot retain the parent, which applied
/// after it; once the consumer is re-run as a successor round after the
/// parent, its latest round retains it.
#[test]
fn a_re_run_consumer_retains_a_parent_appended_after_its_head() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let answers = RefCell::new(None);
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
                let (consumer, parent) = inputs(candidate)?;
                let head = candidate
                    .retain_output_inputs(execution, &consumer, &[&parent])
                    .map_err(refusal);
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
                *answers.borrow_mut() = Some((head, rerun));
                Ok(())
            }),
        )
        .unwrap();
    assert_eq!(
        answers.into_inner(),
        Some((Err(Cause::ForeignCoupledOwner), Ok(())))
    );
}
