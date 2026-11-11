//! Membership and exclusivity include the original Native source-scope custody.
use super::publication_fixture::PublishedModel;
use super::*;
#[test]
fn owned_publication_members_join_the_model_and_refuse_a_second_document_owner() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        world
            .runtime
            .observe_model_owner(execution, &basis, &ownership(), world.output)
            .unwrap(),
        Some(world.model)
    );
    let closure = world
        .runtime
        .prepare_owned_model_entity_boundary(execution, &basis, &basis, &ownership(), world.model)
        .unwrap();
    assert_eq!(
        closure
            .selected_entities()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        [world.model, world.source, world.output]
            .into_iter()
            .collect()
    );
    drop(closure);
    let other = create_entity(execution, &world.runtime, "other-model");
    let (_, before) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let mut tx = begin(&world.runtime);
    tx.push_batch(
        WorkerIntentBatch::new("foreign-ownership").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("foreign-owner"),
                source: EntityReference::Existing(other),
                target: EntityReference::Existing(world.output),
                fields: AspectFieldPatch::default(),
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let error = tx.commit(execution, &world.runtime).unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("{error:?}");
    };
    let ConflictClass::InvariantViolation {
        fields: InvariantViolationFields::CustomInvariantViolation { refusal, .. },
        ..
    } = error.class
    else {
        panic!("{error:?}");
    };
    let worth_relational::facade::runtime::CustomInvariantRefusal::DeclaredValue {
        value_identity,
        bytes,
    } = refusal
    else {
        panic!("{refusal:?}");
    };
    assert_eq!(
        value_identity,
        worth_relational::facade::branch::RelationalModelOwnershipConflict::VALUE_IDENTITY
    );
    let conflict =
        worth_relational::facade::branch::RelationalModelOwnershipConflict::decode(&bytes).unwrap();
    assert_eq!(conflict.subject, world.output);
    assert_eq!(
        world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}
#[test]
fn cold_native_publication_custody_proves_owned_members_without_history_reconstruction() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let (cold, storage) = fixture::cold::reopen(
        execution,
        &world.runtime,
        super::publication_fixture::registry(),
    );
    let (_, basis) = cold.observe_branch(&cold.main_branch_identity()).unwrap();
    assert_eq!(
        cold.observe_model_owner(execution, &basis, &ownership(), world.output)
            .unwrap(),
        Some(world.model)
    );
    assert!(storage.region_read_count() > 0);
    assert_eq!(
        cold.observe_branch(&cold.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
#[test]
fn custody_observation_spends_the_original_requests_work_ceiling() {
    use worth_foundational::facade::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        world
            .runtime
            .observe_model_owner(execution, &basis, &ownership(), world.output)
            .unwrap(),
        Some(world.model)
    );
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), None, Some(64)),
    );
    let bounded = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let error = world
        .runtime
        .observe_model_owner(
            &ExecutionRequest::serial(&bounded),
            &basis,
            &ownership(),
            world.output,
        )
        .unwrap_err();
    assert_eq!(
        error.kernel_stop(),
        Some(worth_execution::MapKernelStop::WorkCeiling)
    );
    assert_eq!(
        world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
