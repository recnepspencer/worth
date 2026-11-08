//! Missing authored catalog membership is never proof that a live route is absent.
use super::*;
use std::ops::ControlFlow;
use worth_relational::facade::{
    content::RelationalContentAttachments, identity::KindId, mvcc::*, transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{
        begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
    },
};
struct Refresh<'a> {
    world: &'a World,
    prior: RelationalRequiredOutputPublication,
}
impl RelationalAuthoredCoupledProducerAssembly for Refresh<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        assert!(
            candidate
                .declared_owner(
                    execution,
                    "source-owned",
                    self.prior.meaning().source_scope,
                    &[0; 32]
                )?
                .is_none(),
            "the source's authored catalog does not contain this other route"
        );
        let running = self
            .world
            .owner
            .admit_required_output("source-owned", [2; 32]);
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
            RequiredOutputPublication::Preserve {
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
            },
            &RelationalContentAttachments::default(),
            vec![
                WorkerIntentBatch::new("refresh-another-current-route").push(
                    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: output,
                            fields: string_aspect_field_patch([(
                                aspect_key("name"),
                                field_key("name"),
                                "refreshed-other",
                            )]),
                        },
                    )),
                ),
            ],
        )?;
        Ok(ControlFlow::Continue(()))
    }
}
#[test]
fn actual_current_custody_admits_a_route_outside_the_authored_catalog() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    let prior = world.add_unrelated_publication(execution);
    let before = world.basis();
    let original = world.read(execution, &before).authoritative_aspect_state;
    let owners = world
        .runtime
        .readmit_edit_coupled_owners(execution, &world.installed, &before, world.source)
        .unwrap();
    let mut tx = begin_main_transaction(&world.runtime);
    tx.push_batch(
        WorkerIntentBatch::new("authored-input").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: world.source,
                fields: string_aspect_field_patch([(
                    aspect_key("name"),
                    field_key("name"),
                    "authored-edit",
                )]),
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let bound = tx
        .bind_authored_coupled_producers(&world.installed, owners, None)
        .unwrap();
    let tx = world
        .runtime
        .preparation_port()
        .prepare_authored_coupled_transaction(
            execution,
            bound,
            &Refresh {
                world: &world,
                prior: prior.clone(),
            },
        )
        .unwrap()
        .unwrap();
    let proposal = tx.validate(execution, &world.runtime).unwrap();
    let candidate = world
        .runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the genuine additional writer must perform")
    };
    let result = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    release_test_commit_snapshot(&world.runtime, &result);
    assert_eq!(
        result.commit.parents,
        [CommitId(
            before
                .reference()
                .target()
                .as_basis()
                .unwrap()
                .selected_commit_id()
        )]
    );
    assert_ne!(
        world
            .read(execution, &world.basis())
            .authoritative_aspect_state,
        original
    );
    let current = world
        .runtime
        .readmit_current_owned_output_successor(
            execution,
            &world.installed,
            &world.basis(),
            "source-owned",
            prior.meaning(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        current.members().next().unwrap().3,
        prior.members().next().unwrap().3
    );
    assert_ne!(commit(&current), commit(&prior));
}
