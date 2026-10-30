#[allow(unused_imports)]
use crate::work_ceiling::isolated_context::retained_run::*;
pub(crate) use {
    worth_execution::ExecutionMap, worth_execution::LeaseRequest, worth_execution::MapKernelFailure,
};

#[test]
fn automatic_sparse_frontier_uses_existing_workers_and_joins_native_settlement() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = crate::work_ceiling::ordinary_context::authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(2).unwrap(),
                    Some(1 << 20),
                    Some(100_000),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let map = ExecutionMap::<_, ()>::from_keyless_partitions(entries(1..=100)).unwrap();
    let worth_execution::RetainedRunOutcome::Complete { run: prior, .. } = map
        .run_retained_run(
            worth_execution::ExecutionRequest::leased(&lease),
            worth_execution::ExecutionWorkCeiling::new(Some(100_000)),
            |value, context| {
                context.checkpoint(5)?;
                Ok::<_, MapKernelFailure<()>>(*value * 10)
            },
        )
        .unwrap()
    else {
        panic!("native initial completion")
    };
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let worth_execution::RetainedRunOutcome::Complete {
        run: next,
        metrics,
        report,
    } = prior
        .advance(
            worth_execution::ExecutionRequest::leased(&lease),
            worth_execution::ExecutionWorkCeiling::new(Some(100_000)),
            entries([2, 99]),
            worth_proof::CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap(),
            |value, context| {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                context.checkpoint(5)?;
                Ok::<_, MapKernelFailure<()>>(*value * 10)
            },
        )
        .unwrap()
    else {
        panic!("native sparse completion")
    };
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
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
        *next
            .result(worth_foundational::PartitionIdentity::new(99))
            .unwrap()
            .value(),
        990
    );
}
