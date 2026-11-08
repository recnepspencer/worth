//! The transaction's C basis is not a producer's pre-writer applied D image.
use super::*;
use std::ops::ControlFlow;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::{
    content::RelationalContentAttachments, identity::KindId, mvcc::*, runtime::*, transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{
        begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
    },
};

struct Echo<'a> {
    world: &'a World,
    prior: RelationalRequiredOutputPublication,
}
impl RelationalAuthoredCoupledProducerAssembly for Echo<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        let source = candidate
            .pin_source(execution)?
            .into_projection_read()
            .project_read()
            .entity_record_with_projection_scope(
                execution,
                self.world.source,
                ProjectionAspectScope::fields(aspect_key("name"), [field_key("name")]),
                |record| {
                    record
                        .aspect_field_value(&aspect_key("name"), &field_key("name"))
                        .cloned()
                },
            )
            .unwrap()
            .expect("the actual applied source");
        assert_eq!(source, AspectValue::String("new-input".into()));
        let AspectValue::String(value) = source else {
            unreachable!()
        };
        let output = self.prior.members().next().unwrap().3;
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
        candidate.append_preserve(
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
                WorkerIntentBatch::new("echo-applied-source").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: output,
                        fields: string_aspect_field_patch([(
                            aspect_key("name"),
                            field_key("name"),
                            value.raw_text().expect("the fixture stores text"),
                        )]),
                    }),
                )),
            ],
        )?;
        Ok(ControlFlow::Continue(()))
    }
}

#[test]
fn a_coupled_producer_cannot_claim_the_transaction_before_basis_as_its_inputs() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    let prior = world.birth(execution);
    let before = world.basis();
    let before_value = world.read(execution, &before).authoritative_aspect_state;
    let owners = world
        .runtime
        .readmit_edit_coupled_owners(execution, &world.installed, &before, world.source)
        .unwrap();
    let mut tx = begin_main_transaction(&world.runtime);
    tx.push_batch(
        WorkerIntentBatch::new("edit-input").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: world.source,
                fields: string_aspect_field_patch([(
                    aspect_key("name"),
                    field_key("name"),
                    "new-input",
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
            &Echo {
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
        panic!("the complete coupled edit must perform")
    };
    let result = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    release_test_commit_snapshot(&world.runtime, &result);
    let selected = world.basis();
    assert_ne!(
        world.read(execution, &selected).authoritative_aspect_state,
        before_value
    );
    let current = world
        .runtime
        .readmit_current_owned_output_successor(
            execution,
            &world.installed,
            &selected,
            "source-owned",
            prior.meaning(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(world.runtime.predecessor_publication_basis(execution, &selected, Some(&current), Some(commit(&current))),
        Err(Denial::PredecessorBasisUnavailable),
        "until Native retains the exact applied input image, it must not supply C as the producer's consumed D");
}
