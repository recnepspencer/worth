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
    expected: &'static str,
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
        assert_eq!(source, AspectValue::String(self.expected.into()));
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
fn coupled_publication_retains_its_actual_applied_inputs() {
    let host = request();
    run(&ExecutionRequest::serial(&host));
}
#[test]
fn worker_only_coupled_publication_retains_its_actual_applied_inputs() {
    use worth_execution::{ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    })
    .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    assert!(!execution.has_accounting());
    run(&execution);
}
fn run(execution: &ExecutionRequest<'_, '_>) {
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
                expected: "new-input",
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
    assert_eq!(
        world.runtime.predecessor_publication_basis(
            execution,
            &selected,
            Some(&current),
            Some(commit(&current))
        ),
        Err(Denial::PredecessorBasisUnavailable),
        "an applied input cannot issue a committed branch basis"
    );
    let input = world
        .runtime
        .predecessor_publication_input(execution, &selected, Some(&current), Some(commit(&current)))
        .unwrap();
    assert_eq!(input.before_basis(), &before);
    let read = input
        .applied_read()
        .expect("the actual coupled pre-writer image");
    assert_eq!(
        read.candidate_identity()
            .qualification()
            .proposed_version_id(),
        result.commit.version_id
    );
    let value = |entity| {
        read.project_candidate()
            .entity_record_with_projection_scope(
                execution,
                entity,
                ProjectionAspectScope::fields(aspect_key("name"), [field_key("name")]),
                |record| {
                    record
                        .aspect_field_value(&aspect_key("name"), &field_key("name"))
                        .cloned()
                },
            )
            .unwrap()
    };
    assert_eq!(
        value(world.source),
        Some(AspectValue::String("new-input".into()))
    );
    assert_eq!(
        value(current.members().next().unwrap().3),
        Some(AspectValue::String("output".into())),
        "inputs precede this writer's own output update"
    );
    world.change_source(execution, "later-current-input");
    assert_eq!(
        value(world.source),
        Some(AspectValue::String("new-input".into()))
    );
    let (cold, restored) = world.reopen(execution, &current);
    assert!(
        matches!(
            cold.runtime.predecessor_publication_input(
                execution,
                &cold.basis(),
                Some(&restored),
                Some(commit(&restored)),
            ),
            Err(Denial::PredecessorBasisUnavailable)
        ),
        "cold recovery cannot reconstruct applied inputs from current truth"
    );
}
