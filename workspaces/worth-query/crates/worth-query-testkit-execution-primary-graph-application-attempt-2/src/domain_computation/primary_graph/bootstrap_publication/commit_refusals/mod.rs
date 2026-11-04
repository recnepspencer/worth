use crate::domain_computation::primary_graph::bootstrap_publication::execution_refusals::authority;
use crate::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated;
use crate::domain_computation::primary_graph::bootstrap_publication::execution_refusals::request;
use std::num::NonZeroUsize;
use worth_execution::CancellationSource;
use worth_execution::LeaseRequest;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as QueryKind;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
use worth_relational::facade::transactions::WorkerIntentBatch;
pub fn refused_commit(
    _execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: LeaseRequest,
) -> worth_relational::facade::transactions::TransactionCommitError {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "bootstrap.entity".to_owned(),
            schema_id: SchemaId("bootstrap-commit-probe".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .expect("valid native runtime configuration");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("bootstrap-commit").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("bootstrap-commit"),
                    fields: AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let lease = authority().request_lease(request).unwrap();
    // Exercise the bootstrap mapper with a real refusal from its native owner.
    runtime
        .prepare_branch_transaction(
            &worth_execution::ExecutionRequest::leased(&lease),
            transaction,
        )
        .expect_err("commit preparation refused")
}
pub fn work_refusal(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> worth_relational::facade::transactions::TransactionCommitError {
    let mut request = request();
    request.policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(0)),
    );
    refused_commit(execution, request)
}
