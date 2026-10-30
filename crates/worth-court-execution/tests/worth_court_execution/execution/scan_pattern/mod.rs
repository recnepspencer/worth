#[allow(unused_imports)]
use crate::scan_pattern::ordinary_context::*;

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
