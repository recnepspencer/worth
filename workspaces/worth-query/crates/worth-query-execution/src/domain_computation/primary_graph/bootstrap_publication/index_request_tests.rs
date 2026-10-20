//! A bootstrap index build consumes the caller's lease and its own stop.
use super::*;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionResourceLease, LeaseRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    indexes::{DerivedIndexDefinition, DerivedIndexExecutionDenialKind, DerivedIndexKind},
    runtime::RelationalRuntimeApi,
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{
        CreateIntent, EntitySpec, MutationIntent, RelationalExecutionDenialCause, WorkerIntentBatch,
    },
};
#[test]
fn bootstrap_index_build_keeps_the_serial_requests_cancellation() {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "index-source".into(),
            schema_id: SchemaId("bootstrap-index-request".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let plain = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let setup = ExecutionRequest::serial(&plain);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("source").push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("source"),
                    fields: Default::default(),
                },
            ))),
            setup.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(&setup, &runtime).unwrap();
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "bootstrap.source-kind".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: worth_foundational::facade::AspectFieldLocator::new(
                worth_foundational::facade::LocatorAuthority::Planned,
                worth_foundational::facade::AspectKey::new("unused").unwrap(),
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("unused").unwrap(),
                ),
            ),
        },
        branch_scoped: true,
    });
    let cancellation = CancellationSource::new();
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(1_000_000)),
        ),
        cancellation: cancellation.token(),
        deadline: None,
    })
    .unwrap();
    cancellation.cancel();
    let build = build_requested_indexes(
        &ExecutionRequest::leased(&lease),
        &runtime,
        DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: worth_relational::facade::history::BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert_eq!(
        build.execution_denial.map(|denial| denial.kind),
        Some(DerivedIndexExecutionDenialKind::Cause(
            RelationalExecutionDenialCause::Cancelled
        ))
    );
}
