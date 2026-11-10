//! Later authored stages apply once each, in Native's admitted order, over
//! the one candidate, before any producer part, and publish in one commit.
use super::*;
use worth_relational::facade::publication::AuthoredRetirementRefusal as Cause;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Shape {
    /// The admitted stage applies, then an owned producer when `mixed`.
    Ordered { mixed: bool },
    /// A stage source Native did not admit.
    Foreign,
    /// The admitted stage never applies.
    Missing,
    /// The admitted stage after a producer part.
    AfterProducer,
    /// The admitted stage rewrites the record the prefix already wrote.
    Rewrite,
    /// The admitted stage links the carrier to the source.
    Link,
}

struct Stages<'a> {
    world: &'a World,
    shape: Shape,
}
impl Stages<'_> {
    fn stage(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        source: EntityId,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        candidate.append_authored_stage(
            execution,
            source,
            &RelationalContentAttachments::default(),
            if self.shape == Shape::Rewrite {
                vec![update(self.world.source, "stage-source")]
            } else if self.shape == Shape::Link {
                vec![
                    WorkerIntentBatch::new("stage-link").push(MutationIntent::Create(
                        CreateIntent::Relation(RelationSpec {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(2),
                            client_key: ClientKey::raw("stage-link"),
                            source: EntityReference::Existing(self.world.carrier),
                            target: EntityReference::Existing(self.world.source),
                            fields: string_aspect_field_patch([(
                                aspect_key("label"),
                                field_key("label"),
                                "stage-link",
                            )]),
                        }),
                    )),
                ]
            } else {
                vec![update(self.world.carrier, "stage-carrier")
                    .push(create_intent("stage-created"))]
            },
        )
    }
    fn producer(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        candidate.append_publication(
            execution,
            RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
                admission: self.world.owner.admit_required_output("owned", [3; 32]),
                absence: Some(self.world.absence(execution, "owned")),
                meaning: self.world.meaning("owned"),
                roles: vec![created("owned-output")],
            }),
            &RelationalContentAttachments::default(),
            vec![create("owned-output")],
        )
    }
}
impl RelationalAuthoredCoupledProducerAssembly for Stages<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        match self.shape {
            Shape::Ordered { mixed } => {
                assert_eq!(candidate.next_authored_stage(), Some(self.world.carrier));
                self.stage(execution, candidate, self.world.carrier)?;
                assert_eq!(candidate.next_authored_stage(), None);
                if mixed {
                    self.producer(execution, candidate)?;
                }
            }
            Shape::Foreign => self.stage(execution, candidate, self.world.source)?,
            Shape::Rewrite | Shape::Link => self.stage(execution, candidate, self.world.carrier)?,
            Shape::Missing => {}
            Shape::AfterProducer => {
                self.producer(execution, candidate)?;
                self.stage(execution, candidate, self.world.carrier)?;
            }
        }
        Ok(ControlFlow::Continue(()))
    }
}

/// The same authored writes as one plain commit: the prefix, then the stage.
fn oracle(execution: &ExecutionRequest<'_, '_>) -> (World, Vec<EntityId>) {
    let world = World::new(execution, true, true);
    let mut tx = begin_main_transaction(&world.runtime);
    for batch in [
        update(world.source, "edited-source"),
        update(world.carrier, "stage-carrier").push(create_intent("stage-created")),
    ] {
        tx.push_batch(batch, execution.allocation_policy()).unwrap();
    }
    let result = tx.commit(execution, &world.runtime).unwrap();
    let created = result.created_entity(&target("stage-created")).unwrap();
    release_test_commit_snapshot(&world.runtime, &result);
    let ids = vec![world.source, world.carrier, created];
    (world, ids)
}

#[test]
fn an_authored_stage_reads_the_candidate_and_commits_with_the_prefix_once() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let (expected_world, expected_ids) = oracle(execution);
    let expected = cold::facts(&expected_world.runtime, execution, &expected_ids);
    for mixed in [false, true] {
        let world = World::new(execution, true, true);
        let parts = Stages {
            world: &world,
            shape: Shape::Ordered { mixed },
        };
        let proposal = world
            .prepare_stages(execution, &[world.carrier], &parts)
            .unwrap();
        assert_eq!(proposal.coupled_participants().count(), usize::from(mixed));
        // Each stage's invariants are judged on its own commands: the stage
        // creates an entity and the prefix only writes a field.
        let creates = |touches: &[ValidatedMutationTouch]| {
            touches
                .iter()
                .any(|touch| matches!(touch, ValidatedMutationTouch::CreateEntity { .. }))
        };
        let stage = proposal
            .authored_stage_mutation_touches(execution, 0)
            .expect("the envelope's one later stage")
            .unwrap();
        assert!(creates(stage.touches()), "{:?}", stage.touches());
        let prefix = proposal.authored_mutation_touches(execution).unwrap();
        assert!(!creates(prefix.touches()), "{:?}", prefix.touches());
        assert!(!prefix.touches().is_empty());
        assert!(proposal
            .authored_stage_mutation_touches(execution, 1)
            .is_none());
        // Its projection examines only its own commands, not the whole proposal.
        let whole = proposal.mutation_touches(execution).unwrap();
        assert!(
            stage.work().validated_intents_examined() < whole.work().validated_intents_examined(),
            "{:?} vs {:?}",
            stage.work(),
            whole.work()
        );
        let result = world.publish(execution, proposal);
        // With an owned participant the canonical event records the prefix's
        // and the stage's sources in order; with none, no coupled event exists.
        assert_eq!(
            result.publication().coupled_authored_stage_sources(),
            mixed.then_some(&[world.source, world.carrier][..])
        );
        // The authored writer's effects are the prefix's and the stage's own
        // patches, never the producer's.
        let authored = result
            .publication()
            .authored_source_effects()
            .expect("the edit envelope retains its authored effects")
            .record_patches()
            .iter()
            .map(|patch| patch.target.clone())
            .collect::<Vec<_>>();
        assert!(authored.contains(&RecordRef::Entity(world.source)));
        assert!(authored.contains(&RecordRef::Entity(world.carrier)));
        assert_eq!(
            authored.len(),
            3,
            "prefix source, stage carrier and stage allocation: {authored:?}"
        );
        let changed = changed_entities(&result);
        release_test_commit_snapshot(&world.runtime, &result);
        // Native scopes each writer's local keys, so the stage's allocation is
        // found among the commit's changes by its committed state.
        let created = changed
            .into_iter()
            .filter(|id| ![world.source, world.carrier].contains(id))
            .find(|id| {
                cold::facts(&world.runtime, execution, &[*id])[0].authoritative_aspect_state
                    == expected[2].authoritative_aspect_state
            })
            .expect("the stage's own allocation commits with the envelope");
        let ids = [world.source, world.carrier, created];
        let actual = cold::facts(&world.runtime, execution, &ids);
        for (a, b) in actual.iter().zip(&expected) {
            assert_eq!(
                a.authoritative_aspect_state, b.authoritative_aspect_state,
                "the prefix and the later stage apply as one commit"
            );
        }
        if mixed {
            // The canonical event records both authored sources and reopens.
            let (cold, _) = cold::reopen(&world, execution, true);
            let reopened = cold::facts(&cold, execution, &ids);
            for (a, b) in reopened.iter().zip(&actual) {
                assert_eq!(a.authoritative_aspect_state, b.authoritative_aspect_state);
            }
        }
    }
}

#[test]
fn stages_outside_the_admitted_roster_refuse_without_effect() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    for (shape, cause) in [
        (Shape::Foreign, Cause::UnadmittedAuthoredStage),
        (Shape::Missing, Cause::MissingAuthoredStage),
        (Shape::AfterProducer, Cause::UnadmittedAuthoredStage),
    ] {
        let before = world.basis();
        let parts = Stages {
            world: &world,
            shape,
        };
        let err = world
            .prepare_stages(execution, &[world.carrier], &parts)
            .unwrap_err();
        let TransactionCommitError::Publication { error, .. } = err else {
            panic!("typed Native stage refusal for {shape:?}: {err:?}")
        };
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(cause),
            "{shape:?}"
        );
        assert_eq!(world.basis(), before, "{shape:?} published nothing");
    }
}

#[test]
fn a_later_stage_cannot_rewrite_a_field_an_earlier_stage_wrote() {
    // A later stage writing the same field of a record an earlier stage wrote
    // conflicts in the one Native commit: it refuses typed, publishing nothing.
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let before = world.basis();
    let parts = Stages {
        world: &world,
        shape: Shape::Rewrite,
    };
    let err = world
        .prepare_stages(execution, &[world.carrier], &parts)
        .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = err else {
        panic!("a typed intent conflict: {err:?}")
    };
    let ConflictClass::ConflictingIntent { target } = &error.class else {
        panic!("a typed intent conflict on the rewritten record: {error:?}")
    };
    // The target type is not on the facade; its Debug names the exact record.
    assert_eq!(format!("{target:?}"), format!("Entity({:?})", world.source));
    assert_eq!(world.basis(), before);
}

/// Publish an owned output whose route is rooted at `source`.
fn birth_at(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    source: EntityId,
    key: &str,
) -> RelationalRequiredOutputPublication {
    let mut meaning = world.meaning("owned");
    meaning.source_scope = source;
    let absence = world
        .runtime
        .readmit_required_output_route_absence(
            execution,
            &world.installed,
            &world.basis(),
            &meaning,
        )
        .unwrap()
        .expect("an actually absent route");
    let mut tx = begin_main_transaction(&world.runtime);
    tx.push_batch(create(key), execution.allocation_policy())
        .unwrap();
    let proposal = tx
        .validate(execution, &world.runtime)
        .unwrap()
        .bind_required_output_publication(
            &world.installed,
            RequiredOutputPublication::Birth {
                admission: world.owner.admit_required_output("owned", [1; 32]),
                absence: Some(absence),
                meaning,
                roles: vec![created(key)],
            },
        )
        .unwrap();
    let result = world.publish(execution, proposal);
    let publication = result
        .required_output_publication(&world.installed)
        .expect("real initial publication");
    release_test_commit_snapshot(&world.runtime, &result);
    publication
}

/// The admitted stage, then a Preserve of a route rooted at the stage source.
struct PreserveAfterStage<'a> {
    world: &'a World,
    prior: RelationalRequiredOutputPublication,
}
impl RelationalAuthoredCoupledProducerAssembly for PreserveAfterStage<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        Stages {
            world: self.world,
            shape: Shape::Ordered { mixed: false },
        }
        .stage(execution, candidate, self.world.carrier)?;
        // Query classifies a route through Native's admitted owner set before
        // running its producer: the stage source's route must be surviving.
        assert!(
            matches!(
                candidate.declared_owner(execution, "owned", self.world.carrier, &[0; 32])?,
                Some(RelationalDeclaredCoupledOwner::Surviving(_))
            ),
            "the stage source's owned route is in the envelope's owner set"
        );
        let running = self.world.owner.admit_required_output("owned", [4; 32]);
        let admission = self
            .world
            .owner
            .admit_required_output_succession(running, self.prior.authority())
            .unwrap();
        let mut meaning = self.prior.meaning().clone();
        meaning.source_epoch = [2; 32];
        let output = self.prior.members().next().unwrap().3;
        candidate.append_publication(
            execution,
            RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Preserve {
                admission,
                predecessor: self.prior.clone(),
                meaning,
                roles: vec![RequiredOutputRoleTarget {
                    role: "carrier-output".into(),
                    entity_name: "node".into(),
                    kind: KindId(1),
                    posture: RequiredOutputRolePosture::Preserve,
                    target: EntityReference::Existing(output),
                }],
            }),
            &RelationalContentAttachments::default(),
            vec![update(output, "preserved-carrier-output")],
        )?;
        Ok(ControlFlow::Continue(()))
    }
}

#[test]
fn a_stage_sources_owned_route_is_an_admitted_writer_of_the_envelope() {
    // Native admits the union of every stage source's current owners, so a
    // producer of a route rooted at a later stage's source may write in the
    // same envelope; it is no foreign participant.
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = birth_at(&world, execution, world.carrier, "carrier-output");
    let proposal = world
        .prepare_stages(
            execution,
            &[world.carrier],
            &PreserveAfterStage {
                world: &world,
                prior,
            },
        )
        .expect("the stage source's owned route is an admitted surviving writer");
    assert_eq!(proposal.coupled_participants().count(), 1);
    let result = world.publish(execution, proposal);
    assert_eq!(
        result
            .required_output_publications(&world.installed, execution)
            .count(),
        1
    );
    release_test_commit_snapshot(&world.runtime, &result);
}

#[test]
fn a_stage_relation_is_part_of_the_authored_relation_footprint() {
    // Authored relation preimage capture reads only the authored writer's
    // relation changes; a stage's link is the authored writer's, not a
    // producer's, so the proposal's authored footprint must carry it.
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let parts = Stages {
        world: &world,
        shape: Shape::Link,
    };
    let proposal = world
        .prepare_stages(execution, &[world.carrier], &parts)
        .unwrap();
    let footprint = proposal
        .relation_mutation_footprint(execution, &[KindId(2)], &[], &[])
        .unwrap();
    let links = footprint
        .relations()
        .iter()
        .map(|relation| (relation.before(), relation.after()))
        .collect::<Vec<_>>();
    assert_eq!(links, [(None, Some((world.carrier, world.source)))]);
}
