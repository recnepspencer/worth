//! An actual Initial publication reads the edited candidate and commits with it.
use super::*;
use std::ops::ControlFlow;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::{
    content::RelationalContentAttachments, identity::*, mvcc::*, runtime::*, symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{
        begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
    },
};
struct Initial<'a>(&'a World);
impl RelationalAuthoredCoupledProducerAssembly for Initial<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        let world = self.0;
        let value = candidate
            .pin_source(execution)?
            .project_candidate()
            .entity_record_with_projection_scope(
                execution,
                world.source,
                ProjectionAspectScope::fields(aspect_key("name"), [field_key("name")]),
                |record| {
                    record
                        .aspect_field_value(&aspect_key("name"), &field_key("name"))
                        .cloned()
                },
            )
            .unwrap()
            .expect("live applied source");
        assert_eq!(value, AspectValue::String("initial-edit".into()));
        let AspectValue::String(text) = value else {
            unreachable!()
        };
        let target = CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw("initial-output"),
        };
        candidate.append_publication(
            execution,
            worth_relational::facade::mvcc::RelationalCoupledProducerPosture::Publication(
                RequiredOutputPublication::Birth {
                    admission: world.owner.admit_required_output("source-owned", [1; 32]),
                    absence: None,
                    meaning: RequiredOutputPublicationMeaning {
                        implementation: RequiredOutputProducingImplementation::from_checkpoint(
                            [0x11; 16], [0x22; 32],
                        )
                        .unwrap(),
                        family: "source-owned".into(),
                        source_scope: world.source,
                        source_partition: [0; 32],
                        source_epoch: [1; 32],
                    },
                    roles: vec![RequiredOutputRoleTarget {
                        role: "output".into(),
                        entity_name: "node".into(),
                        kind: KindId(1),
                        posture: RequiredOutputRolePosture::Create,
                        target: EntityReference::Created(target),
                    }],
                },
            ),
            &RelationalContentAttachments::default(),
            vec![
                WorkerIntentBatch::new("initial-derived-output").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: ClientKey::raw("initial-output"),
                        fields: string_aspect_field_patch([(
                            aspect_key("name"),
                            field_key("name"),
                            text.raw_text().expect("fixture text"),
                        )]),
                    }),
                )),
            ],
        )?;
        Ok(ControlFlow::Continue(()))
    }
}
fn run(execution: &ExecutionRequest<'_, '_>) {
    let world = World::new(execution);
    let before = world.basis();
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
                    "initial-edit",
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
        .prepare_authored_coupled_transaction(execution, bound, &Initial(&world))
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
        panic!("the complete edit must perform")
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
    let selected = world.basis();
    let meaning = RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: "source-owned".into(),
        source_scope: world.source,
        source_partition: [0; 32],
        source_epoch: [1; 32],
    };
    let current = world
        .runtime
        .readmit_current_owned_output_successor(
            execution,
            &world.installed,
            &selected,
            "source-owned",
            &meaning,
        )
        .unwrap()
        .unwrap();
    let members = current.members().collect::<Vec<_>>();
    assert_eq!(members.len(), 1);
    assert_eq!(
        (members[0].0, members[0].1, members[0].2),
        ("output", "node", RequiredOutputRolePosture::Create)
    );
    let output = members[0].3;
    assert_ne!(output, world.source);
    let input = world
        .runtime
        .predecessor_publication_input(execution, &selected, Some(&current), Some(commit(&current)))
        .unwrap();
    assert_eq!(input.before_basis(), &before);
    let read = input.applied_read().expect("actual Initial inputs");
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
        Some(AspectValue::String("initial-edit".into()))
    );
    assert_eq!(
        value(output),
        None,
        "this writer's new output was absent from its inputs"
    );
    let (cold, restored) = world.reopen(execution, &current);
    assert_eq!(restored.members().next().unwrap().3, output);
    assert!(matches!(
        cold.runtime.predecessor_publication_input(
            execution,
            &cold.basis(),
            Some(&restored),
            Some(commit(&restored))
        ),
        Err(Denial::PredecessorBasisUnavailable)
    ));
}
#[test]
fn initial_output_and_authored_edit_are_one_native_publication() {
    let host = request();
    run(&ExecutionRequest::serial(&host));
}
#[test]
fn worker_only_initial_output_and_authored_edit_are_one_native_publication() {
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

#[test]
fn a_birth_cannot_replace_a_live_route_even_with_the_installed_owner() {
    use worth_relational::facade::publication::AuthoredRetirementRefusal;
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    let prior = world.birth(execution);
    let before = world.basis();
    let value = world.read(execution, &before).authoritative_aspect_state;
    let owners = world
        .runtime
        .readmit_edit_coupled_owners(execution, &world.installed, &before, world.source)
        .unwrap();
    let mut tx = begin_main_transaction(&world.runtime);
    tx.push_batch(
        WorkerIntentBatch::new("refused-authored-input").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: world.source,
                fields: string_aspect_field_patch([(
                    aspect_key("name"),
                    field_key("name"),
                    "initial-edit",
                )]),
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let bound = tx
        .bind_authored_coupled_producers(&world.installed, owners, None)
        .unwrap();
    let denied = world
        .runtime
        .preparation_port()
        .prepare_authored_coupled_transaction(execution, bound, &Initial(&world))
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = denied else {
        panic!("{denied:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::ForeignCoupledOwner)
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(world.source)]
    );
    assert_eq!(world.basis(), before);
    assert_eq!(
        world
            .read(execution, &world.basis())
            .authoritative_aspect_state,
        value
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
    assert!(current.same_origin(&prior));
}
