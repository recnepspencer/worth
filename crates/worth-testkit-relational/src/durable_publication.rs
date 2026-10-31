//! Public owner-admitted publications and recovery over the original test medium.
use crate::aspect_storage::{
    publications::{begin_main_transaction, release_test_commit_snapshot},
    AspectSchemaFixture,
};
use crate::durable_medium::TestDurableMedium;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::{
    branch::RecordedRelationalComponentReference as Recorded,
    config::{CascadeDeletePolicy, RelationalRuntimeProfile},
    durability::{RecoveredRelationalRuntimeAuthority, RecoveryVerificationMode},
    identity::{KindId, PartitionId},
    mvcc::{
        PerformedRelationalCommit, RelationalCallerAssociation as Association,
        RelationalPublicationOutcome,
    },
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    transactions::{
        planned_single_field_locator, AspectFieldPatch, CommitResult, CreateIntent, EntitySpec,
        MutationIntent, WorkerIntentBatch,
    },
};

pub fn schema() -> worth_relational::facade::schema::RelationalSchemaRegistry {
    AspectSchemaFixture::with_default_declared_aspects(CascadeDeletePolicy::CascadeDeleteRelations)
        .build_registry()
}
pub fn durable_runtime(medium: &Arc<TestDurableMedium>) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .build_durable(medium.attachment())
        .unwrap()
}
pub fn memory_runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .build()
        .unwrap()
}
pub fn field_patch(aspect: &str, field: &str, value: &str) -> AspectFieldPatch {
    AspectFieldPatch::from_locator(
        planned_single_field_locator(
            AspectKey::new(aspect).unwrap(),
            FieldKey::new(field).unwrap(),
        ),
        AspectValue::String(value.into()),
    )
}
pub fn batch_create(key: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(format!("batch-{key}")).push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: key.into(),
            fields: field_patch("name", "name", key),
        }),
    ))
}
pub fn create_entity_outcome(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}
pub fn create_entity(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, key: &str) {
    let result = create_entity_outcome(execution, runtime, key);
    crate::aspect_storage::publications::release_test_commit_snapshot(runtime, &result);
}
pub fn current(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) -> Recorded {
    let port = runtime.owner_component_services().basis_port();
    let (_, basis) = port
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    port.export_recorded_current_component(execution, &basis)
        .unwrap()
}
pub fn perform(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    association: &Association,
    key: &str,
) -> PerformedRelationalCommit {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("actual source effect performs")
    };
    performed
}
pub fn recover(
    execution: &ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
) -> (RelationalRuntime, RecoveredRelationalRuntimeAuthority) {
    let mut runtime = durable_runtime(medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    (runtime, authority)
}

mod checkpoint;
mod recorded_assertions;
mod recovery;
pub use checkpoint::{capture, original, original_with_retained_successor, Original};
pub use recorded_assertions::assert_recorded_basis;
pub use recovery::source_journey;
