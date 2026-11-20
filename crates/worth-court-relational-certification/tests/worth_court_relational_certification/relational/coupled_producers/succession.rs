//! #119 PR A: an owned route's later round is a complete re-derivation of its
//! chain. Native folds it in place, so the commit equals the one-round answer,
//! or refuses with its typed #119 reason, publishing nothing.
use super::*;
use worth_relational::facade::publication::{
    AuthoredRetirementRefusal as Cause, CoupledProvenancePostponement as Reason,
};

type Candidate<'a> = RelationalAuthoredCoupledCandidate<'a>;
type Failure = RelationalCoupledAssemblyFailure;

struct Assemble<F>(F);
impl<F> RelationalAuthoredCoupledProducerAssembly for Assemble<F>
where
    F: Fn(&ExecutionRequest<'_, '_>, &mut Candidate<'_>) -> Result<(), Failure>,
{
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut Candidate<'_>,
    ) -> Result<ControlFlow<()>, Failure> {
        (self.0)(execution, candidate)?;
        Ok(ControlFlow::Continue(()))
    }
}
fn assemble<F>(rounds: F) -> Assemble<F>
where
    F: Fn(&ExecutionRequest<'_, '_>, &mut Candidate<'_>) -> Result<(), Failure>,
{
    Assemble(rounds)
}

fn named(key: &str, value: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
        fields: string_aspect_field_patch([(aspect_key("name"), field_key("name"), value)]),
    }))
}
fn creates(keys: &[&str], value: &str) -> Vec<WorkerIntentBatch> {
    vec![keys
        .iter()
        .fold(WorkerIntentBatch::new("round"), |batch, key| {
            batch.push(named(key, &format!("{key}:{value}")))
        })]
}
fn birth(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    keys: &[&str],
) -> RelationalCoupledProducerPosture {
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
        admission: world.owner.admit_required_output("owned", [3; 32]),
        absence: Some(world.absence(execution, "owned")),
        meaning: world.meaning("owned"),
        roles: keys.iter().map(|key| created(key)).collect(),
    })
}
fn birth_roles(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    roles: Vec<RequiredOutputRoleTarget>,
) -> RelationalCoupledProducerPosture {
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
        admission: world.owner.admit_required_output("owned", [3; 32]),
        absence: Some(world.absence(execution, "owned")),
        meaning: world.meaning("owned"),
        roles,
    })
}
fn preserve(
    world: &World,
    prior: &RelationalRequiredOutputPublication,
) -> RelationalCoupledProducerPosture {
    preserve_as(world, prior, RequiredOutputRolePosture::Preserve)
}
fn preserve_as(
    world: &World,
    prior: &RelationalRequiredOutputPublication,
    posture: RequiredOutputRolePosture,
) -> RelationalCoupledProducerPosture {
    let running = world.owner.admit_required_output("owned", [4; 32]);
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Preserve {
        admission: world
            .owner
            .admit_required_output_succession(running, prior.authority())
            .unwrap(),
        predecessor: prior.clone(),
        meaning: preserve_meaning(prior),
        roles: vec![RequiredOutputRoleTarget {
            role: "output".into(),
            entity_name: "node".into(),
            kind: KindId(1),
            posture,
            target: EntityReference::Existing(output(prior)),
        }],
    })
}
fn preserve_meaning(
    prior: &RelationalRequiredOutputPublication,
) -> RequiredOutputPublicationMeaning {
    let mut meaning = prior.meaning().clone();
    meaning.source_epoch = worth_foundational::CanonicalDigestId::new([2; 32]);
    meaning
}
fn output(prior: &RelationalRequiredOutputPublication) -> EntityId {
    prior.members().next().unwrap().3
}
fn none() -> RelationalContentAttachments {
    RelationalContentAttachments::default()
}
/// One successor round of the route's chain, through its continuation token.
fn successor(
    execution: &ExecutionRequest<'_, '_>,
    candidate: &mut Candidate<'_>,
    posture: RelationalCoupledProducerPosture,
    batches: Vec<WorkerIntentBatch>,
) -> Result<(), Failure> {
    let token = candidate.next_producer_round(execution, 0)?;
    candidate.append_successor_publication(execution, token, posture, &none(), batches)
}
fn refusal(err: TransactionCommitError) -> Cause {
    let TransactionCommitError::Publication { error, .. } = err else {
        panic!("a typed Native refusal: {err:?}")
    };
    error.authored_retirement_refusal().expect("an authored refusal")
}
fn postponed(reason: Reason) -> Cause {
    Cause::CoupledContinuationProvenancePostponed { issue: 119, reason }
}
/// Everything a commit publishes that the one-round answer fixes.
#[derive(Debug, PartialEq)]
struct Observed {
    /// Source, carrier and the route's outputs, read back.
    states: Vec<String>,
    changed: usize,
    /// Counted from the commit's canonical deltas.
    aspects: String,
    /// The published record patches.
    patches: String,
    /// The route's validated touches, from its recorded commands.
    touches: String,
}
/// Publish the one-chain proposal and observe it.
fn committed(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    proposal: ValidatedRelationalProposal,
    meaning: &RequiredOutputPublicationMeaning,
) -> (Observed, Vec<EntityId>) {
    assert_eq!(proposal.coupled_participants().count(), 1, "one chain, one participant");
    let touches = format!(
        "{:?}",
        proposal
            .coupled_producer_mutation_touches(execution, meaning)
            .expect("the route's touches")
            .unwrap()
            .touches()
    );
    let result = world.publish(execution, proposal);
    let origins = result
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(origins.len(), 1, "one route publishes one origin");
    let mut ids = vec![world.source, world.carrier];
    ids.extend(origins[0].members().map(|member| member.3));
    let observed = Observed {
        states: states(&cold::facts(&world.runtime, execution, &ids)),
        changed: changed_entities(&result).len(),
        aspects: format!("{:?}", result.aspect_summary()),
        patches: format!(
            "{:?}",
            result.publication().envelope.patch.authoritative_record_patches
        ),
        touches,
    };
    release_test_commit_snapshot(&world.runtime, &result);
    (observed, ids)
}
#[derive(Clone, Copy)]
enum Route<'k> {
    /// A Birth of these keys.
    Birth(&'k [&'k str]),
    /// A Preserve of a real earlier publication's output.
    Preserve,
}
/// The route's head round, then one successor round per later value, in
/// `world`. `round` authors a value's batches over the Preserve output.
fn chain(
    execution: &ExecutionRequest<'_, '_>,
    world: &World,
    route: Route<'_>,
    values: &[&str],
    round: &dyn Fn(&str, Option<EntityId>) -> Vec<WorkerIntentBatch>,
) -> (Observed, Vec<EntityId>) {
    let prior = matches!(route, Route::Preserve)
        .then(|| world.birth(execution, "owned", "owned-output"));
    let target = prior.as_ref().map(output);
    let meaning = prior
        .as_ref()
        .map_or_else(|| world.meaning("owned"), preserve_meaning);
    let birth_route = matches!(route, Route::Birth(_));
    let posture = |execution: &ExecutionRequest<'_, '_>| match route {
        Route::Birth(keys) => birth(world, execution, keys),
        Route::Preserve => preserve(world, prior.as_ref().unwrap()),
    };
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    posture(execution),
                    &none(),
                    round(values[0], target),
                )?;
                let inventory = |candidate: &mut Candidate<'_>| {
                    candidate
                        .appended_output_entities(execution, "owned", &meaning)
                        .map(|outputs| outputs.expect("the appended route").to_vec())
                };
                let before = if birth_route {
                    inventory(candidate)?
                } else {
                    Vec::new()
                };
                for value in &values[1..] {
                    successor(execution, candidate, posture(execution), round(value, target))?;
                    // Inventory: the public readers see the latest round on
                    // the head's own allocations.
                    if birth_route {
                        assert_eq!(inventory(candidate)?, before, "no second allocation");
                        assert!(candidate.applied_output_input(execution, &meaning)?.is_some());
                    }
                }
                Ok(())
            }),
        )
        .unwrap();
    if birth_route {
        let contribution = proposal
            .coupled_contribution_records("owned", &meaning)
            .unwrap();
        assert!(contribution.publication_required());
    }
    committed(world, execution, proposal, &meaning)
}
fn summary_update(entity: EntityId, title: &str, status: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("summary-write").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: entity,
            fields: string_aspect_field_patch([
                (aspect_key("summary"), field_key("title"), title),
                (aspect_key("summary"), field_key("status"), status),
            ]),
        }),
    ))
}
fn summary_creates(keys: &[&str], value: &str) -> Vec<WorkerIntentBatch> {
    vec![keys
        .iter()
        .fold(WorkerIntentBatch::new("round"), |batch, key| {
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(*key),
                fields: string_aspect_field_patch([
                    (aspect_key("name"), field_key("name"), *key),
                    (aspect_key("summary"), field_key("title"), value),
                    (aspect_key("summary"), field_key("status"), value),
                ]),
            })))
        })]
}
/// A portable field patch of the Struct aspect `summary`.
fn summary_patch(entity: EntityId, fields: &[(&str, &str)]) -> WorkerIntentBatch {
    use worth_foundational::facade::{
        PortableAspectContractBasis, PortableAspectFieldSet, PortableAspectPatchOperation,
        PortableRecordAspectPatch,
    };
    let (_, contract) =
        worth_testkit_relational::aspect_storage::definition::entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        );
    let keys = fields
        .iter()
        .map(|(field, _)| field_key(field))
        .collect::<Vec<_>>();
    let sets = fields
        .iter()
        .map(|(field, value)| {
            PortableAspectFieldSet::new(
                field_key(field),
                worth_testkit_relational::aspect_storage::publications::text(value),
            )
        })
        .collect();
    WorkerIntentBatch::new("summary-patch").push(MutationIntent::Entity(
        EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
            entity_id: entity,
            aspect_patch: PortableRecordAspectPatch::new([
                PortableAspectPatchOperation::PatchFields {
                    basis: PortableAspectContractBasis::from_contract(&contract),
                    selected_fields: keys,
                    field_sets: sets,
                    field_clears: Vec::new(),
                },
            ]),
        }),
    ))
}
fn states(
    facts: &[worth_relational::facade::runtime::EntityReadRecord],
) -> Vec<String> {
    facts
        .iter()
        .map(|fact| format!("{:?}", fact.authoritative_aspect_state))
        .collect()
}

#[test]
fn a_preserve_round_folds_to_the_one_round_answer_and_reopens() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let write = |value: &str, output: Option<EntityId>| vec![update(output.unwrap(), value)];
    let run = |values: &[&str]| {
        let world = World::new(execution, true, true);
        let (observed, ids) = chain(execution, &world, Route::Preserve, values, &write);
        (world, observed, ids)
    };
    let (world, actual, ids) = run(&["round-1", "round-2"]);
    assert_eq!(actual, run(&["round-2"]).1);
    let (cold, _) = cold::reopen(&world, execution, true);
    assert_eq!(states(&cold::facts(&cold, execution, &ids)), actual.states);
    // C -> b -> C nets to what one round writing C publishes.
    assert_eq!(
        run(&["round-1", "owned-output"]).1,
        run(&["owned-output"]).1,
        "a chain back to the base value"
    );
}

#[test]
fn struct_aspect_rounds_fold_to_the_one_round_answer() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let keys = ["owned-output", "owned-second"];
    let birth = |values: &[&str]| {
        let world = World::new_with_summary(execution);
        let round = |value: &str, _: Option<EntityId>| summary_creates(&keys, value);
        chain(execution, &world, Route::Birth(&keys), values, &round).0
    };
    assert_eq!(
        birth(&["round-1", "round-2", "round-3"]),
        birth(&["round-3"]),
        "Birth"
    );
    let preserve = |values: &[&str]| {
        let world = World::new_with_summary(execution);
        let round = |value: &str, output: Option<EntityId>| {
            vec![summary_update(output.unwrap(), value, value)]
        };
        chain(execution, &world, Route::Preserve, values, &round).0
    };
    assert_eq!(
        preserve(&["round-1", "round-2", "round-3"]),
        preserve(&["round-3"]),
        "Preserve"
    );
}

#[test]
fn aspect_patch_rounds_fold_and_cover() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let patch = |values: &[&str]| {
        let world = World::new_with_summary(execution);
        let round = |value: &str, output: Option<EntityId>| {
            vec![summary_patch(
                output.unwrap(),
                &[("title", value), ("status", value)],
            )]
        };
        chain(execution, &world, Route::Preserve, values, &round).0
    };
    assert_eq!(patch(&["round-1", "round-2"]), patch(&["round-2"]));
    let world = World::new_with_summary(execution);
    let prior = world.birth(execution, "owned", "owned-output");
    let target = output(&prior);
    let head = vec![summary_patch(target, &[("title", "a"), ("status", "a")])];
    let rounds = |round: Vec<WorkerIntentBatch>| {
        let (world, prior, head) = (&world, &prior, &head);
        assemble(move |execution, candidate| {
            candidate.append_publication(execution, preserve(world, prior), &none(), head.clone())?;
            successor(execution, candidate, preserve(world, prior), round.clone())
        })
    };
    // A field update of the same fields covers an earlier aspect patch.
    world
        .prepare(execution, &rounds(vec![summary_update(target, "b", "b")]))
        .unwrap();
    // A patch of fewer fields leaves the earlier patch uncovered.
    let before = world.basis();
    let err = world
        .prepare(
            execution,
            &rounds(vec![summary_patch(target, &[("title", "b")])]),
        )
        .unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::UncoveredWrite));
    assert_eq!(world.basis(), before);
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
        prepared
            .map(|proposal| committed(&world, execution, proposal, &preserve_meaning(&prior)).0)
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
fn a_birth_round_folds_onto_its_own_allocation() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let keys = ["owned-output", "owned-second"];
    let run = |values: &[&str]| {
        let world = World::new(execution, true, true);
        let round = |value: &str, _: Option<EntityId>| creates(&keys, value);
        chain(execution, &world, Route::Birth(&keys), values, &round).0
    };
    assert_eq!(run(&["round-1", "round-2"]), run(&["round-2"]));
}

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
                    vec![WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
                        EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: fresh }),
                    ))],
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
        let other = RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
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
        ("stale", &stale as &dyn RelationalAuthoredCoupledProducerAssembly, Cause::StaleCoupledContinuation),
        ("other posture kind", &other_kind, Cause::RepeatedCoupledOwner),
        ("second first entry", &second_entry, Cause::RepeatedCoupledOwner),
    ] {
        let before = world.basis();
        let err = world.prepare(execution, assembly).unwrap_err();
        assert_eq!(refusal(err), cause, "{case}");
        assert_eq!(world.basis(), before, "{case}");
    }
}

#[test]
fn a_folded_chain_costs_one_round_at_two_widths() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    for width in [4usize, 16] {
        let keys = (0..width).map(|index| format!("owned-{index}")).collect::<Vec<_>>();
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
            (work, committed(&world, execution, proposal, &world.meaning("owned")).0)
        };
        assert_eq!(
            examined(&["round-1", "round-2", "round-3"]),
            examined(&["round-3"]),
            "width {width}: the folded chain validates and publishes one round"
        );
    }
}
