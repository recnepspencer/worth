//! A later authored stage contributes its real source-owned producer route.
use super::*;

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
        meaning.source_epoch = worth_foundational::CanonicalDigestId::new([2; 32]);
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
