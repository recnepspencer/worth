//! A bootstrap index build consumes the caller's lease and its own stop.
use worth_execution::CancellationSource;
use worth_execution::ExecutionRequest;
use worth_execution::ExecutionResourceLease;
use worth_execution::LeaseRequest;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_query_execution::domain_computation::primary_graph::bootstrap_publication::build_requested_indexes;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::indexes::DerivedIndexDefinition;
use worth_relational::facade::indexes::DerivedIndexExecutionDenialKind;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::indexes::DerivedIndexKind;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
use worth_relational::facade::transactions::WorkerIntentBatch;
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
        .build()
        .expect("valid native runtime configuration");
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
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
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
        })
        .expect("index fixture has identity capacity");
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
