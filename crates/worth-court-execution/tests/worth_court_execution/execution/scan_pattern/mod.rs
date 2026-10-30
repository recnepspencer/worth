#[allow(unused_imports)]
use crate::scan_pattern::ordinary_context::*;
pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    std::sync::Mutex, std::sync::OnceLock,
};

pub(crate) use {
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionScan, worth_execution::LeaseDenial, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapStop,
    worth_execution::ScanOutcome,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

#[test]
fn failed_step_keeps_only_the_committed_prefix() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outcome = scan().run(None, 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        if *item == 2 {
            return Err(worth_execution::MapKernelFailure::Domain("invalid"));
        }
        let next = *carry + *item;
        Ok((next, next))
    });
    assert!(matches!(outcome, worth_execution::ScanOutcome::Stopped {
        completed_state: 1,
        completed_prefix,
        boundary: Some(identity),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Domain("invalid"), .. },
        ..
    } if completed_prefix == vec![1] && identity == worth_foundational::PartitionIdentity::new(2)));
}
