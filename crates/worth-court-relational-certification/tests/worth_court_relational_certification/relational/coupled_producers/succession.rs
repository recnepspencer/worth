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
fn preserve(
    world: &World,
    prior: &RelationalRequiredOutputPublication,
) -> RelationalCoupledProducerPosture {
    let running = world.owner.admit_required_output("owned", [4; 32]);
    let mut meaning = prior.meaning().clone();
    meaning.source_epoch = worth_foundational::CanonicalDigestId::new([2; 32]);
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Preserve {
        admission: world
            .owner
            .admit_required_output_succession(running, prior.authority())
            .unwrap(),
        predecessor: prior.clone(),
        meaning,
        roles: vec![RequiredOutputRoleTarget {
            role: "output".into(),
            entity_name: "node".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Preserve,
            target: EntityReference::Existing(output(prior)),
        }],
    })
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
/// Publish and read back the source, carrier and the route's outputs.
fn committed(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    proposal: ValidatedRelationalProposal,
) -> (Vec<worth_relational::facade::runtime::EntityReadRecord>, usize) {
    assert_eq!(proposal.coupled_participants().count(), 1, "one chain, one participant");
    let result = world.publish(execution, proposal);
    let origins = result
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(origins.len(), 1, "one route publishes one origin");
    let changed = changed_entities(&result).len();
    let mut ids = vec![world.source, world.carrier];
    ids.extend(origins[0].members().map(|member| member.3));
    release_test_commit_snapshot(&world.runtime, &result);
    (cold::facts(&world.runtime, execution, &ids), changed)
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
    let oracle = World::new(execution, true, true);
    let prior = oracle.birth(execution, "owned", "owned-output");
    let proposal = oracle
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    preserve(&oracle, &prior),
                    &none(),
                    vec![update(output(&prior), "round-2")],
                )
            }),
        )
        .unwrap();
    let (expected, expected_changed) = committed(&oracle, execution, proposal);

    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let proposal = world
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
                    vec![update(output(&prior), "round-2")],
                )
            }),
        )
        .unwrap();
    let (actual, changed) = committed(&world, execution, proposal);
    assert_eq!(states(&actual), states(&expected));
    assert_eq!(changed, expected_changed, "one change per record");
    let (cold, _) = cold::reopen(&world, execution, true);
    let ids = [world.source, world.carrier, output(&prior)];
    assert_eq!(states(&cold::facts(&cold, execution, &ids)), states(&actual));
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
        states(&committed(&world, execution, proposal).0)
    };
    assert_eq!(run(&["round-1", "round-2", "round-3"]), run(&["round-3"]));
}

#[test]
fn a_birth_round_folds_onto_its_own_allocation() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let keys = ["owned-output", "owned-second"];
    let oracle = World::new(execution, true, true);
    let proposal = oracle
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    birth(&oracle, execution, &keys),
                    &none(),
                    creates(&keys, "round-2"),
                )
            }),
        )
        .unwrap();
    let (expected, expected_changed) = committed(&oracle, execution, proposal);

    let world = World::new(execution, true, true);
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_publication(
                    execution,
                    birth(&world, execution, &keys),
                    &none(),
                    creates(&keys, "round-1"),
                )?;
                let meaning = world.meaning("owned");
                let before = candidate
                    .appended_output_entities(execution, "owned", &meaning)?
                    .expect("the appended route")
                    .to_vec();
                successor(
                    execution,
                    candidate,
                    birth(&world, execution, &keys),
                    creates(&keys, "round-2"),
                )?;
                // Inventory: the public readers see the chain's latest round
                // on the head's own allocations.
                let after = candidate
                    .appended_output_entities(execution, "owned", &meaning)?
                    .expect("the folded route")
                    .to_vec();
                assert_eq!(after, before, "no second allocation");
                assert!(candidate.applied_output_input(execution, &meaning)?.is_some());
                Ok(())
            }),
        )
        .unwrap();
    let contribution = proposal
        .coupled_contribution_records("owned", &world.meaning("owned"))
        .unwrap();
    assert!(contribution.publication_required());
    let (actual, changed) = committed(&world, execution, proposal);
    assert_eq!(states(&actual), states(&expected));
    assert_eq!(changed, expected_changed, "the chain allocated once");
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
            let (facts, changed) = committed(&world, execution, proposal);
            (work, changed, states(&facts))
        };
        assert_eq!(
            examined(&["round-1", "round-2", "round-3"]),
            examined(&["round-3"]),
            "width {width}: the folded chain validates and publishes one round"
        );
    }
}
