pub(crate) use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};

pub(crate) use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionScan, LeaseDenial, LeaseRequest, MapKernelFailure, MapKernelStop, MapStop,
    ScanOutcome,
};
pub(crate) use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: Some(4096),
        })
        .unwrap()
    })
}

pub(crate) fn lease(
    work: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), Some(4096), Some(work)),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

pub(crate) fn scan() -> ExecutionScan<u64> {
    let ids = [1, 2, 3].map(PartitionIdentity::new);
    ExecutionScan::try_from_ordered(ids.to_vec(), ids.into_iter().zip([1, 2, 3]).collect()).unwrap()
}

#[test]
fn failed_step_keeps_only_the_committed_prefix() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outcome = scan().run(None, 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        if *item == 2 {
            return Err(MapKernelFailure::Domain("invalid"));
        }
        let next = *carry + *item;
        Ok((next, next))
    });
    assert!(matches!(outcome, ScanOutcome::Stopped {
        completed_state: 1,
        completed_prefix,
        boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Domain("invalid"), .. },
        ..
    } if completed_prefix == vec![1] && identity == PartitionIdentity::new(2)));
}
