//! Owner row application follows packet staging in the same original Work account.
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    runtime::RelationalWorkAccountRecorderForTest,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, BulkEntityCreateIntent, BulkRelationCreateIntent,
        CommitExecutionDenialKind, CommitResult, CreateIntent, EntityReference, MutationIntent,
        TransactionCommitError, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_publication::{create_entity_outcome, field_patch, memory_runtime},
};

const FUNDED: u64 = 8_000_000;
#[derive(Clone, Copy, Debug)]
enum Rows {
    Entities,
    Relations,
}

#[test]
fn bulk_owner_rows_share_the_original_account_after_packet_staging() {
    // One process authority funds every trial; each trial owns a fresh lease.
    let authority = worth_execution::ExecutionAuthority::try_construct(
        worth_execution::ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::MIN,
            charged_memory_bytes: Some(512 * 1024 * 1024),
        },
    )
    .unwrap();
    let trial = |rows, ceiling, wrapped| trial(&authority, rows, ceiling, wrapped);
    for rows in [Rows::Entities, Rows::Relations] {
        let (reference, required) = trial(rows, FUNDED, true);
        reference.unwrap();
        assert!(required > 0 && required < FUNDED);
        for wrapped in [false, true] {
            let (outcome, actual) = trial(rows, required, wrapped);
            outcome.unwrap_or_else(|cause| panic!("{rows:?}: exact original invoice: {cause:?}"));
            assert_eq!(
                actual, required,
                "{rows:?}: all owner and packet work settles"
            );
        }
        let (refused, _) = trial(rows, required - 1, false);
        assert!(
            matches!(refused, Err(TransactionCommitError::Execution {denial,..})
            if denial.kind == CommitExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted)),
            "{rows:?}: one below the actual complete invoice refuses"
        );
    }
}

fn trial(
    authority: &worth_execution::ExecutionAuthority,
    rows: Rows,
    ceiling: u64,
    wrapped: bool,
) -> (Result<CommitResult, TransactionCommitError>, u64) {
    let setup = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let setup_execution = ExecutionRequest::serial(&setup);
    let runtime = memory_runtime();
    let seed = |name| {
        let result = create_entity_outcome(&setup_execution, &runtime, name);
        let entity = changed_entities(&result)[0];
        release_test_commit_snapshot(&runtime, &result);
        entity
    };
    let source = seed("source");
    let target = seed("target");
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(ceiling)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let intent = match rows {
        Rows::Entities => {
            MutationIntent::Create(CreateIntent::BulkEntities(BulkEntityCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_keys: vec![ClientKey::raw("bulk-a"), ClientKey::raw("bulk-b")],
                field_patches: ["bulk-a", "bulk-b"]
                    .into_iter()
                    .map(|name| field_patch("name", "name", name))
                    .collect(),
            }))
        }
        Rows::Relations => {
            MutationIntent::Create(CreateIntent::BulkRelations(BulkRelationCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_keys: vec![ClientKey::raw("edge")],
                endpoints: vec![(
                    EntityReference::Existing(source),
                    EntityReference::Existing(target),
                )],
                field_patches: vec![AspectFieldPatch::default()],
            }))
        }
    };
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
            WorkerIntentBatch::new("owned-bulk-work").push(intent),
            execution.allocation_policy(),
        )
        .unwrap();
    let run = || {
        RelationalWorkAccountRecorderForTest::new()
            .record_for_test(|| transaction.commit(&execution, &runtime))
    };
    let ((outcome, charges), whole) = if wrapped {
        let (outcome, report) = execution
            .run(ExecutionWorkCeiling::new(None), |_| run())
            .unwrap();
        (outcome, Some(report.charged_work()))
    } else {
        (run(), None)
    };
    let settled = charges.into_iter().sum();
    if let Some(whole) = whole {
        assert_eq!(
            settled, whole,
            "{rows:?}: the same invocation accounts every SDK charge"
        );
    }
    if outcome.is_err() {
        assert_eq!(
            runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0,
            before,
            "original Work refusal publishes no successor"
        );
    }
    (outcome, whole.unwrap_or(settled))
}
