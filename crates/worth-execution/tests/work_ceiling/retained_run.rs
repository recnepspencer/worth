use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};
use worth_execution::{
    CancellationToken, ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, KeylessPartition,
    LeaseRequest, MapKernelFailure, RetainedRunOutcome,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};
use worth_proof::CanonicalUniqueVec;

fn entries(
    ids: impl IntoIterator<Item = u64>,
) -> BTreeMap<PartitionIdentity, KeylessPartition<u64>> {
    ids.into_iter()
        .map(|id| {
            (
                PartitionIdentity::new(id),
                KeylessPartition {
                    value: id,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )
        })
        .collect()
}

#[test]
fn automatic_sparse_frontier_uses_existing_workers_and_joins_native_settlement() {
    let authority = super::authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), Some(1 << 20), Some(100_000)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let map = ExecutionMap::<_, ()>::from_keyless_partitions(entries(1..=100)).unwrap();
    let RetainedRunOutcome::Complete { run: prior, .. } = map
        .run_retained_run(
            ExecutionRequest::leased(&lease),
            ExecutionWorkCeiling::new(Some(100_000)),
            |value, context| {
                context.checkpoint(5)?;
                Ok::<_, MapKernelFailure<()>>(*value * 10)
            },
        )
        .unwrap()
    else {
        panic!("native initial completion")
    };
    let calls = AtomicUsize::new(0);
    let RetainedRunOutcome::Complete {
        run: next,
        metrics,
        report,
    } = prior
        .advance(
            ExecutionRequest::leased(&lease),
            ExecutionWorkCeiling::new(Some(100_000)),
            entries([2, 99]),
            CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap(),
            |value, context| {
                calls.fetch_add(1, Ordering::SeqCst);
                context.checkpoint(5)?;
                Ok::<_, MapKernelFailure<()>>(*value * 10)
            },
        )
        .unwrap()
    else {
        panic!("native sparse completion")
    };
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        (
            next.partition_count(),
            next.charged_work(),
            next.charged_span()
        ),
        (100, 500, 5)
    );
    assert_eq!((report.charged_work(), report.charged_span()), (500, 5));
    assert_eq!(
        (
            metrics.supplied_inputs,
            metrics.carried_slots,
            metrics.carried_slot_visits
        ),
        (2, 98, 0)
    );
    assert!(
        metrics.range_visits < 70 && metrics.copied_nodes < 80,
        "{metrics:?}"
    );
    assert_eq!(
        *next.result(PartitionIdentity::new(99)).unwrap().value(),
        990
    );
}
