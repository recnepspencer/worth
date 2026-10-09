//! Native staged index/footprint custody and semantic atomic refusal.
//! Input/nested key/value, normalization and rollback output heaps remain excluded.
use crate::{facade::mvcc::RelationalTransactionIntent, tests::support::*};
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialKind,
    ExecutionAllocationPolicy as Policy, LeaseDenial, LeaseRequest, MemoryLimitDenial,
    MemoryLimitLevel,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

const BUDGET: u64 = 128 * 1024;
fn request(memory: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
#[test]
fn native_staging_allocation_refusal_and_shared_footprint_custody() {
    let authority = test_execution_authority();
    let parent = authority.request_lease(request(BUDGET)).unwrap();
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    {
        let child = parent.child(request(BUDGET)).unwrap();
        transaction
            .push_batch(
                batch_create("retained-native-created"),
                Policy::Execution(&child),
            )
            .unwrap();
    }
    let retained = transaction.footprint().clone();
    assert_eq!(retained.writes().len(), 1);
    let crate::facade::mvcc::RelationalTransactionWriteLocus::CreatedEntity(key) =
        retained.writes().next().unwrap()
    else {
        panic!("actual created footprint")
    };
    assert_eq!(key.client_key.as_raw_str(), Some("retained-native-created"));
    assert!(
        parent.reserve_memory(BUDGET).is_err(),
        "native backing outlives its child lease"
    );
    let savepoint = transaction.create_savepoint().unwrap();
    let cached = transaction
        .merged_plan(&runtime, Policy::Execution(&parent))
        .unwrap()
        .clone();
    let before_batches = transaction.batches().to_vec();
    let before_footprint = transaction.footprint().clone();
    let zero = parent.child(request(0)).unwrap();
    let failure = transaction
        .push_batch(batch_create("must-not-stage"), Policy::Execution(&zero))
        .unwrap_err();
    let cause = failure
        .allocation_denial()
        .expect("original physical cause retained");
    // CreateEntity first adds a new write locus: Author::CHUNK is 32
    // RefCell<Option<WriteLocus>> slots; no existing row has this raw key.
    let staging_bytes = (32
        * std::mem::size_of::<
            std::cell::RefCell<Option<crate::facade::mvcc::RelationalTransactionWriteLocus>>,
        >()) as u64;
    assert_eq!(
        cause.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(MemoryLimitDenial {
            requested: staging_bytes,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        }))
    );
    assert!(
        cause
            .requested_payload_bytes()
            .is_some_and(|bytes| bytes > 0),
        "owner quote retained, no guessed aggregate budget"
    );
    assert_eq!(transaction.batches(), before_batches);
    assert_eq!(transaction.footprint(), &before_footprint);
    assert_eq!(transaction.last_merged_plan.as_ref(), Some(&cached));
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
    drop(zero);
    let cancellation = CancellationSource::new();
    let mut cancelled_request = request(BUDGET);
    cancelled_request.cancellation = cancellation.token();
    let cancelled = parent.child(cancelled_request).unwrap();
    cancellation.cancel();
    let stopped = transaction
        .push_batch(
            batch_create("cancelled-native-created"),
            Policy::Execution(&cancelled),
        )
        .unwrap_err();
    let cause = stopped.allocation_denial().unwrap();
    assert_eq!(cause.kind(), ExecutionAllocationDenialKind::Cancelled);
    assert_eq!(
        cause.requested_payload_bytes(),
        None,
        "stop precedes layout admission"
    );
    assert_eq!(transaction.batches(), before_batches);
    assert_eq!(transaction.footprint(), &before_footprint);
    assert_eq!(transaction.last_merged_plan.as_ref(), Some(&cached));
    drop(cancelled);
    transaction
        .push_batch(
            batch_create("later-native-created"),
            Policy::Execution(&parent),
        )
        .unwrap();
    assert_eq!(transaction.batches().len(), 2);
    transaction.rollback_to_savepoint(savepoint).unwrap();
    assert_eq!(transaction.batches().len(), 1);
    assert_eq!(transaction.footprint(), &retained);
    assert_eq!(
        transaction
            .merged_plan(&runtime, Policy::Execution(&parent),)
            .unwrap()
            .merged_intents
            .len(),
        1
    );
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
    drop((transaction, before_footprint, cached, before_batches));
    assert!(
        parent.reserve_memory(BUDGET).is_err(),
        "retained footprint still owns native backing"
    );
    drop(retained);
    drop(
        parent
            .reserve_memory(BUDGET)
            .expect("last retained footprint releases backing"),
    );
}
