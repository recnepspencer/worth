//! A real preparation refusal shared by the workspace and effect boundary proofs.
use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, TransactionCommitError,
        WorkerIntentBatch,
    },
};

pub(crate) fn work_exhausted() -> TransactionCommitError {
    static OWNER: OnceLock<ExecutionAuthority> = OnceLock::new();
    let owner = OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::MIN,
            charged_memory_bytes: 64 * 1024 * 1024,
        })
        .unwrap()
    });
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "carriage.entity".to_owned(),
            schema_id: SchemaId("carriage-probe".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("carriage").push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("carriage"),
                    fields: AspectFieldPatch::default(),
                },
            ))),
        )
        .unwrap();
    let lease = owner
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024, 0),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let error = runtime
        .prepare_branch_transaction_with_lease(transaction, &lease)
        .expect_err("real owner refusal");
    let TransactionCommitError::Execution { denial, .. } = &error else {
        panic!("execution refused");
    };
    assert_eq!(
        denial.kind,
        worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted
        )
    );
    assert_eq!(denial.partition_identity, Some(1));
    error
}
