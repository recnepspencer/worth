use super::*;
use std::time::Instant;

fn assert_request_cause(request: LeaseRequest, expected: fn(Cause) -> bool) {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();
    let (created, _) = support::create_entity(&runtime, "cause-source");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "cause-source.name".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: support::aspect_field_locator("name"),
        },
        branch_scoped: true,
    });
    let lease = authority()
        .request_lease(request)
        .expect("request policy is admitted");
    let index_outcome = runtime.index_authority().build_for_commit_with_lease(
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
        &lease,
    );
    let DerivedIndexExecutionDenialKind::Cause(index_cause) = index_outcome
        .execution_denial
        .expect("index execution stops")
        .kind;
    assert!(expected(index_cause), "index cause: {index_cause:?}");
    assert!(index_outcome.generations.is_empty());

    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).expect("current basis");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("basis admits transaction");
    transaction
        .push_batch(
            WorkerIntentBatch::new("cause-target").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("cause-target"),
                    fields: AspectFieldPatch::default(),
                }),
            )),
            AllocationPolicy::SystemAllocation,
        )
        .expect("intent stages");
    let error = runtime
        .prepare_branch_transaction_with_lease(transaction, &lease)
        .expect_err("commit execution stops");
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("wrong commit stop: {error:?}");
    };
    let CommitExecutionDenialKind::Cause(commit_cause) = denial.kind;
    assert!(expected(commit_cause), "commit cause: {commit_cause:?}");
    assert_eq!(
        runtime
            .observe_branch(&identity)
            .expect("main remains current")
            .0,
        before
    );
}

#[test]
fn cancelled_lease_preserves_both_request_causes() {
    let _serial = serial();
    let source = worth_execution::CancellationSource::new();
    source.cancel();
    assert_request_cause(lease_request(32 * 1024 * 1024, source.token()), |cause| {
        cause == Cause::Cancelled
    });
}

#[test]
fn expired_lease_preserves_both_request_causes() {
    let _serial = serial();
    let mut request = lease_request(32 * 1024 * 1024, CancellationToken::new());
    request.deadline = Some(Instant::now());
    assert_request_cause(request, |cause| cause == Cause::DeadlineElapsed);
}

#[test]
fn exhausted_work_lease_preserves_both_request_causes() {
    let _serial = serial();
    assert_request_cause(
        lease_request_with_work(32 * 1024 * 1024, CancellationToken::new(), 0),
        |cause| cause == Cause::WorkExhausted,
    );
}

#[test]
fn exhausted_policy_memory_preserves_both_request_causes() {
    let _serial = serial();
    assert_request_cause(lease_request(0, CancellationToken::new()), |cause| {
        matches!(
            cause,
            Cause::PolicyMemoryExhausted {
                requested: 1..,
                admitted: 0,
                ancestor: 0
            }
        )
    });
}

#[test]
fn exhausted_process_memory_preserves_both_request_causes() {
    let _serial = serial();
    let held = authority()
        .request_lease(lease_request(64 * 1024 * 1024, CancellationToken::new()))
        .expect("competing request is admitted");
    let _reservation = held
        .reserve_memory(64 * 1024 * 1024)
        .expect("process memory is held");
    assert_request_cause(
        lease_request(32 * 1024 * 1024, CancellationToken::new()),
        |cause| {
            matches!(
                cause,
                Cause::ProcessMemoryExhausted {
                    requested: 1..,
                    admitted: 0
                }
            )
        },
    );
}
