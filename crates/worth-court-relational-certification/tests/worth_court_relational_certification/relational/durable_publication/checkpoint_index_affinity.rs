use super::checkpoint_index_generations::{fork_branch, scoped_name_index};
use super::checkpoint_recovery_observation::snapshot_for_branch;
use super::checkpoint_source_capture::capture_durably;
use super::merge_execution_observation::entity_field_display;
use std::sync::Arc;
use worth_relational::facade::{
    durability::{RecoveryPlan, RecoveryVerificationMode},
    history::BranchId,
    identity::{EntityId, KindId},
    indexes::{
        BoundedEntityFieldLookupRequest, BoundedIndexParityMode, DerivedIndexBuildRequest,
        DerivedIndexGenerationId, DerivedIndexId,
    },
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    aspect_storage::{
        definition::{aspect_field_locator, aspect_key, field_key},
        publications::{changed_entities, release_test_commit_snapshot},
    },
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};
const NAME: &str = "scoped-checkpoint-shared-root";
const FORK: &str = "scoped-checkpoint-real-fork";

fn fork_checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
) -> (
    RecoveryPlan,
    EntityId,
    DerivedIndexId,
    DerivedIndexGenerationId,
) {
    let runtime = durable_runtime(medium);
    let anchor = create_entity_outcome(execution, &runtime, NAME);
    let entity = changed_entities(&anchor)[0];
    let branch = fork_branch(execution, &runtime, FORK);
    assert_ne!(anchor.commit.branch_id, branch);
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    assert_eq!(
        basis.observation().commit_id(),
        Some(anchor.commit.commit_id)
    );
    let index = scoped_name_index(&runtime);
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch,
            index_ids: vec![index.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    assert_eq!(built.generations.len(), 1);
    let generation = built.generations[0].generation_id;
    drop(basis);
    release_test_commit_snapshot(&runtime, &anchor);
    capture_durably(execution, &runtime, medium).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    (plan, entity, index.index_id, generation)
}

fn name_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity: EntityId,
) -> Option<String> {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let read = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap()
        .read_entity(execution, entity)
        .unwrap();
    read.base()
        .and_then(|record| entity_field_display(record, &aspect_key("name"), &field_key("name")))
}

#[test]
fn real_shared_parent_root_scoped_checkpoint_restores_lookup_and_content() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let (plan, entity, index, generation) = fork_checkpoint(execution, &medium);
    let mut restored = durable_runtime(&medium);
    restored
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let branch = BranchId(FORK.into());
    let identity = restored.branch_identity(&branch).unwrap();
    let (_, basis) = restored.observe_branch(&identity).unwrap();
    let envelope = restored
        .replay()
        .canonical_commit_envelope(execution, basis.observation().commit_id().unwrap())
        .unwrap()
        .unwrap();
    assert_ne!(&envelope.commit.branch_id, &branch);
    let admitted = restored
        .index_access()
        .read_published_generation_for_observation(execution, index, &basis.observation())
        .unwrap()
        .unwrap();
    assert_eq!(admitted.generation_id, generation);
    assert_eq!(admitted.source_branch_id, branch);
    let snapshot = snapshot_for_branch(&restored, &branch);
    let request = || {
        BoundedEntityFieldLookupRequest::new(
            snapshot.clone(),
            index,
            KindId(1),
            aspect_field_locator(aspect_key("name"), field_key("name")),
            worth_foundational::facade::AspectValue::String(NAME.into()),
            2,
        )
        .unwrap()
    };
    let actual = restored
        .index_access()
        .execute_bounded_entity_field_lookup(
            request(),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap();
    assert_eq!(actual.generation_id(), generation);
    assert_eq!(actual.candidate_entity_ids(), &[entity]);
    let parity = restored
        .index_access()
        .execute_bounded_entity_field_lookup(
            request(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    assert_eq!(parity.candidate_entity_ids(), &[entity]);
    assert_eq!(
        name_on_branch(execution, &restored, &branch, entity),
        Some(NAME.into())
    );
    restored.snapshots().release_snapshot(&snapshot).unwrap();
}
