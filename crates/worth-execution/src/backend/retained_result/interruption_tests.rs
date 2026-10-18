use super::*;
use crate::{
    ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, KeylessPartition, SerialMemoryBudget,
    SerialRequest,
};
use std::collections::BTreeMap;

/// Cancellation happens during the native capacity examination, after the
/// kernel's final checkpoint. No instrumentation door or extra poll is needed.
struct CancelWhenSized<'a>(&'a crate::CancellationSource);
impl ChargedBytes for CancelWhenSized<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.cancel();
        0
    }
}

#[test]
fn interruption_observed_after_inner_settlement_cannot_be_swallowed_by_the_invoking_kernel() {
    let cancellation = crate::CancellationSource::new();
    let memory = SerialMemoryBudget::new(100_000);
    let parent = SerialRequest::from_memory(memory.clone(), crate::CancellationToken::new(), None);
    let child = parent.clone().with_cancellation(cancellation.token());
    let map = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: (),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        },
    )]))
    .unwrap();
    let outer =
        crate::backend::scope::run_scope_within(None, 0, 0, true, 100, Some(&parent), |context| {
            let refused = map.run_retained(
                ExecutionRequest::serial(&child),
                ExecutionWorkCeiling::new(100),
                |_, _| Ok::<_, KernelFailure<()>>(CancelWhenSized(&cancellation)),
            );
            assert!(matches!(
                refused,
                Err(crate::RetainedMapDenial::Request(
                    crate::WorkCeilingDenial::Stopped(KernelStop::Cancelled)
                ))
            ));
            assert_eq!(
                context.take_nested_computation_stop(),
                None,
                "a request fact cannot be consumed"
            );
            // Intentionally swallow the returned error and make no further poll.
            Ok::<_, KernelFailure<()>>(())
        });
    assert!(matches!(
        outer.result,
        Err(crate::backend::scope::ScopeStop::Failure(
            KernelFailure::Stop(KernelStop::NestedStopped)
        ))
    ));
    assert!(memory.reserve(100_000).is_ok());
}

#[test]
fn cancellation_at_the_last_combine_releases_the_rejected_tree_target_before_fresh_retry() {
    let cancellation = crate::CancellationSource::new();
    let memory = SerialMemoryBudget::new(100_000);
    let parent = SerialRequest::from_memory(memory.clone(), crate::CancellationToken::new(), None);
    let child = parent.clone().with_cancellation(cancellation.token());
    let map = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: 7_u64,
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        },
    )]))
    .unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let mut target = memory.reserve(0).unwrap();
    let outcome = map.run_reduce_retained_holding(
        ExecutionRequest::serial(&child),
        ExecutionWorkCeiling::new(100),
        None,
        Some(&mut target),
        |input, context| {
            context.checkpoint(1)?;
            Ok::<_, KernelFailure<()>>(*input)
        },
        0_u64,
        |left, right| {
            if calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 1 {
                cancellation.cancel();
            }
            left + right
        },
        1_000,
        0,
    );
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::Relaxed),
        2,
        "actual last canonical combine"
    );
    assert!(matches!(
        outcome.result,
        Err(crate::RetainedReduceDenial::Retention(
            crate::RetainedMapDenial::Request(crate::WorkCeilingDenial::Stopped(
                KernelStop::Cancelled
            ))
        ))
    ));
    assert_eq!(
        outcome.reduction_metrics.unwrap().combine_calls,
        2,
        "completed native work stays observed"
    );
    assert_eq!(
        target.bytes(),
        0,
        "a canceled completed tree publishes no retained target"
    );
    assert!(memory.reserve(100_000).is_ok());
    let fresh = map.run_reduce_retained_holding(
        ExecutionRequest::serial(&parent),
        ExecutionWorkCeiling::new(100),
        None,
        Some(&mut target),
        |input, context| {
            context.checkpoint(1)?;
            Ok::<_, KernelFailure<()>>(*input)
        },
        0_u64,
        |a, b| a + b,
        1_000,
        0,
    );
    let fresh = fresh.result.unwrap();
    assert_eq!(*fresh.tree.result(), 7);
    assert!(target.bytes() > 0);
    let prior_bytes = target.bytes();
    let foreign = SerialRequest::from_memory(
        SerialMemoryBudget::new(100_000),
        crate::CancellationToken::new(),
        None,
    );
    let entered = std::sync::atomic::AtomicUsize::new(0);
    let refused = map.run_reduce_retained_holding(
        ExecutionRequest::serial(&foreign),
        ExecutionWorkCeiling::new(100),
        None,
        Some(&mut target),
        |input, _| {
            entered.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok::<_, KernelFailure<()>>(*input)
        },
        0_u64,
        |a, b| a + b,
        1_000,
        0,
    );
    assert!(matches!(
        refused.result,
        Err(crate::RetainedReduceDenial::Reduction(
            crate::ReduceInputDenial::ScopeAdmission { .. }
        ))
    ));
    assert_eq!(entered.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(
        target.bytes(),
        prior_bytes,
        "a refused foreign takeover keeps the original tree's grant"
    );
    assert_eq!(
        *fresh.tree.result(),
        7,
        "the genuinely retained old tree stays usable"
    );
    drop(fresh);
    target.resize(0).unwrap();
    assert!(memory.reserve(100_000).is_ok());
}
