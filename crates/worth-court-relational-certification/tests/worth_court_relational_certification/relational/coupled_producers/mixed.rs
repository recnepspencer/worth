//! Mixed Ordinary and Preserve writers keep one owned lineage and real ordinary rows.
use super::*;
struct Mixed<'a> {
    world: &'a World,
    prior: RelationalRequiredOutputPublication,
}
impl RelationalAuthoredCoupledProducerAssembly for Mixed<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        if assembly(self.world)
            .assemble(execution, candidate)?
            .is_break()
        {
            return Ok(ControlFlow::Break(()));
        }
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
                roles: self
                    .prior
                    .members()
                    .map(|(role, entity, _, id)| RequiredOutputRoleTarget {
                        role: role.into(),
                        entity_name: entity.into(),
                        kind: KindId(1),
                        posture: RequiredOutputRolePosture::Preserve,
                        target: EntityReference::Existing(id),
                    })
                    .collect(),
            }),
            &RelationalContentAttachments::default(),
            vec![update(output, "preserved-owned-output")],
        )?;
        Ok(ControlFlow::Continue(()))
    }
}
#[test]
fn ordinary_and_preserved_writers_keep_only_the_real_owned_lineage_after_reopen() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let prior = world.birth(execution, "owned", "owned-output");
    let prior_id = prior.members().next().unwrap().3;
    let proposal = world
        .prepare(
            execution,
            &Mixed {
                world: &world,
                prior,
            },
        )
        .unwrap();
    assert_eq!(proposal.coupled_participants().count(), 2);
    let contribution = proposal
        .coupled_contribution_records("ordinary", &world.meaning("ordinary"))
        .unwrap();
    assert!(!contribution.publication_required());
    let companion = contribution
        .ordinary_output_members()
        .unwrap()
        .find(|(role, ..)| *role == "ordinary-companion")
        .unwrap()
        .3;
    let result = world.publish(execution, proposal);
    let origins = result
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0].producer(), "owned");
    assert_eq!(origins[0].members().next().unwrap().3, prior_id);
    release_test_commit_snapshot(&world.runtime, &result);
    let ids = [world.source, world.carrier, companion, prior_id];
    let expected = cold::facts(&world.runtime, execution, &ids);
    let (reopened, installed) = cold::reopen(&world, execution, true);
    let actual = cold::facts(&reopened, execution, &ids);
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(&expected) {
        assert_eq!(a.authoritative_aspect_state, b.authoritative_aspect_state);
    }
    let basis = reopened
        .observe_branch(&reopened.main_branch_identity())
        .unwrap()
        .1;
    for (family, absent) in [("ordinary", true), ("owned", false)] {
        assert_eq!(
            reopened
                .readmit_required_output_route_absence(
                    execution,
                    &installed,
                    &basis,
                    &world.meaning(family),
                )
                .unwrap()
                .is_some(),
            absent
        );
    }
}
