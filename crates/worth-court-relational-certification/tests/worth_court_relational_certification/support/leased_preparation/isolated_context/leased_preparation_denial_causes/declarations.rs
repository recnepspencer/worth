pub(crate) use {worth_execution::ExecutionAllocationPolicy as AllocationPolicy, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::history::BranchId, worth_execution::CancellationToken, worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause, worth_relational::facade::symbols::ClientKey, worth_relational::facade::transactions::CommitExecutionDenialKind, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::indexes::DerivedIndexBuildRequest, worth_relational::facade::indexes::DerivedIndexDefinition, worth_relational::facade::indexes::DerivedIndexExecutionDenialKind, worth_relational::facade::indexes::DerivedIndexId, worth_relational::facade::indexes::DerivedIndexKind, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::identity::KindId, worth_execution::LeaseRequest, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::identity::PartitionId, worth_relational::facade::runtime::RelationalRuntimeApi, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch, crate::leased_preparation::ordinary_context::authority, crate::leased_preparation::ordinary_context::lease_request, crate::leased_preparation::ordinary_context::lease_request_with_work, crate::leased_preparation::ordinary_context::serial, crate::leased_preparation::ordinary_context::support};
pub(crate) use {std::time::Instant};

pub(crate) fn assert_request_cause(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: LeaseRequest,
    expected: fn(Cause) -> bool,
) {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(crate::leased_preparation::ordinary_context::support::demo_schema_registry())
        .build()
        .expect("valid native runtime configuration");
    let (created, _) = crate::leased_preparation::ordinary_context::support::create_entity(execution, &runtime, "cause-source");
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "cause-source.name".to_owned(),
            kind: worth_relational::facade::indexes::DerivedIndexKind::EntityField {
                field_locator: crate::leased_preparation::ordinary_context::support::aspect_field_locator("name"),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
    let lease = authority()
        .request_lease(request)
        .expect("request policy is admitted");
    let index_outcome = runtime.index_authority().build_for_commit(
        &worth_execution::ExecutionRequest::leased(&lease),
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
    );
    let worth_relational::facade::indexes::DerivedIndexExecutionDenialKind::Cause(index_cause) = index_outcome
        .execution_denial
        .expect("index execution stops")
        .kind;
    assert!(expected(index_cause), "index cause: {index_cause:?}");
    assert!(index_outcome.generations.is_empty());

    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).expect("current basis");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .expect("basis admits transaction");
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new("cause-target").push(worth_relational::facade::transactions::MutationIntent::Create(
                worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: worth_relational::facade::symbols::ClientKey::raw("cause-target"),
                    fields: worth_relational::facade::transactions::AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("intent stages");
    let error = runtime
        .prepare_branch_transaction(
            &worth_execution::ExecutionRequest::leased(&lease),
            transaction,
        )
        .expect_err("commit execution stops");
    let worth_relational::facade::transactions::TransactionCommitError::Execution { denial, .. } = error else {
        panic!("wrong commit stop: {error:?}");
    };
    let worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(commit_cause) = denial.kind;
    assert!(expected(commit_cause), "commit cause: {commit_cause:?}");
    assert_eq!(
        runtime
            .observe_branch(&identity)
            .expect("main remains current")
            .0,
        before
    );
}