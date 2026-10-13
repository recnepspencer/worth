//! Empty requests cannot materialize rows or publish a commit at their native door.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexExecutionDenialKind,
    DerivedIndexId, DerivedIndexKind,
};
use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};
use crate::visibility::materialization::read_records::materialization::row_reads_for_test::RowReadsForTest;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
const MEMORY: u64 = 64 * 1024 * 1024;
fn lease(memory: u64, work: u64) -> worth_execution::ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, work),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}
fn fixture() -> (RelationalRuntime, crate::transactions::data::CommitResult) {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&request, &runtime, "read-custody");
    (runtime, created)
}
#[test]
fn empty_read_request_refuses_before_rows() {
    let (runtime, created) = fixture();
    let entity = changed_entities(&created)[0];
    let plan = planned_explicit_query(
        &runtime,
        &created.snapshot,
        "empty-read",
        vec![RecordRef::Entity(entity)],
    );
    let rows = RowReadsForTest::install_on_this_thread();
    for (memory, work) in [(MEMORY, 0), (0, u64::MAX)] {
        let lease = lease(memory, work);
        let stopped = runtime
            .read_truth()
            .execute_query_plan(plan.clone(), &ExecutionRequest::leased(&lease))
            .unwrap_err();
        let CommitExecutionDenialKind::Cause(cause) = stopped.execution_denial().unwrap().kind;
        assert_empty_cause(memory, cause);
        assert_eq!(rows.rows(), 0);
    }
    let admitted = lease(MEMORY, u64::MAX);
    runtime
        .read_truth()
        .execute_query_plan(plan, &ExecutionRequest::leased(&admitted))
        .unwrap()
        .unwrap();
    assert_eq!(
        rows.rows(),
        1,
        "the counted reader observes an admitted row"
    );
}
#[test]
fn empty_index_request_keeps_its_cause_and_publishes_no_generation() {
    let (runtime, created) = fixture();
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(7),
        name: "empty-request".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let build_request = || DerivedIndexBuildRequest {
        source_commit_id: created.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index.index_id],
    };
    for (memory, work) in [(MEMORY, 0), (0, u64::MAX)] {
        let lease = lease(memory, work);
        let result = runtime.index_authority().build_for_basis(
            &ExecutionRequest::leased(&lease),
            build_request(),
            &basis,
        );
        let DerivedIndexExecutionDenialKind::Cause(cause) = result.execution_denial.unwrap().kind;
        assert_empty_cause(memory, cause);
        assert!(
            result.generations.is_empty(),
            "a refused build publishes no generation"
        );
        assert_eq!(result.failed_indexes, vec![index.index_id]);
    }
    let admitted = lease(MEMORY, u64::MAX);
    let built = runtime.index_authority().build_for_basis(
        &ExecutionRequest::leased(&admitted),
        build_request(),
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    assert_eq!(
        built.generations.len(),
        1,
        "the admitted build publishes its requested index"
    );
}
#[test]
fn empty_commit_request_refuses_before_publication() {
    let (runtime, _) = fixture();
    let before = runtime.history().latest_commit().unwrap().commit_id;
    let transaction = || {
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                batch_create("commit-custody"),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        transaction
    };
    for (memory, work) in [(MEMORY, 0), (0, u64::MAX)] {
        let lease = lease(memory, work);
        let stopped = runtime
            .commit_branch_transaction(&ExecutionRequest::leased(&lease), transaction())
            .unwrap_err();
        let TransactionCommitError::Execution { denial, .. } = stopped else {
            panic!("native execution cause required: {stopped:?}");
        };
        let CommitExecutionDenialKind::Cause(cause) = denial.kind;
        assert_empty_cause(memory, cause);
        let after = runtime.history().latest_commit().unwrap().commit_id;
        assert_eq!(after.0 - before.0, 0, "a refused commit publishes no head");
    }
    let admitted = lease(MEMORY, u64::MAX);
    let committed = runtime
        .commit_branch_transaction(&ExecutionRequest::leased(&admitted), transaction())
        .unwrap();
    let after = runtime.history().latest_commit().unwrap().commit_id;
    assert_eq!(after, committed.commit.commit_id);
    assert_eq!(
        after.0 - before.0,
        1,
        "the same admitted door publishes one head"
    );
}

fn assert_empty_cause(memory: u64, cause: Cause) {
    if memory != 0 {
        assert_eq!(cause, Cause::WorkExhausted);
    } else {
        assert!(matches!(
            cause,
            Cause::PolicyMemoryExhausted {
                admitted: 0,
                ancestor: 0,
                ..
            }
        ));
    }
}
