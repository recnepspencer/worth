//! Fresh admission is the changed native path, not a second grant for history.
use std::{collections::BTreeMap, sync::atomic::Ordering};
use worth_execution::{
    CancellationToken, ExecutionMap, ExecutionRequest, ExecutionRetainedRun, ExecutionWorkCeiling,
    KeylessPartition, MapKernelContext, MapKernelFailure, RetainedRunOutcome, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;
fn serial(memory: &SerialMemoryBudget) -> SerialRequest {
    SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None)
}
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
fn empty_ids() -> CanonicalUniqueVec<PartitionIdentity> {
    CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap()
}
fn kernel(v: &u64, c: &mut MapKernelContext<'_, '_>) -> Result<u64, MapKernelFailure<()>> {
    for units in [2, 0, 3] {
        c.checkpoint(units)?;
    }
    Ok(v * 10)
}
fn complete(outcome: RetainedRunOutcome<u64, ()>) -> ExecutionRetainedRun<u64> {
    match outcome {
        RetainedRunOutcome::Complete { run, .. } => run,
        _ => panic!("native complete"),
    }
}
fn seed(count: u64, request: &SerialRequest) -> ExecutionRetainedRun<u64> {
    let map = ExecutionMap::<_, ()>::from_keyless_partitions(entries(1..=count)).unwrap();
    complete(
        map.run_retained_run(
            ExecutionRequest::serial(request),
            ExecutionWorkCeiling::new(Some(100_000)),
            kernel,
        )
        .unwrap(),
    )
}
use std::sync::atomic::AtomicU64;
fn delta(count: u64) -> u64 {
    let prior_memory = SerialMemoryBudget::new(None);
    let original = serial(&prior_memory);
    let prior = seed(count, &original);
    let prior_held = prior_memory.charged_memory_bytes();
    let fresh_memory = SerialMemoryBudget::new(None);
    let fresh = serial(&fresh_memory);
    let at_kernel = AtomicU64::new(0);
    let next = complete(
        prior
            .advance(
                ExecutionRequest::serial(&fresh),
                ExecutionWorkCeiling::new(Some(100_000)),
                entries([count]),
                empty_ids(),
                |value, context| {
                    at_kernel.store(fresh_memory.charged_memory_bytes(), Ordering::SeqCst);
                    kernel(value, context)
                },
            )
            .unwrap(),
    );
    assert_eq!(
        next.result(PartitionIdentity::new(count)).unwrap().value(),
        &(count * 10)
    );
    assert_eq!(next.charged_work(), prior.charged_work());
    assert_eq!(prior_memory.charged_memory_bytes(), prior_held);
    let admitted = at_kernel
        .load(Ordering::SeqCst)
        .max(fresh_memory.charged_memory_bytes());
    let retained = next.clone();
    drop(next);
    drop(prior);
    drop(original);
    // Shared cells still own their original admission after the source handle
    // is gone. Both budgets release independently on the last actual cell.
    assert!(prior_memory.charged_memory_bytes() > 0);
    assert!(fresh_memory.charged_memory_bytes() > 0);
    drop(retained);
    assert_eq!(prior_memory.charged_memory_bytes(), 0);
    assert_eq!(fresh_memory.charged_memory_bytes(), 0);
    admitted
}
#[test]
fn fixed_native_advance_admits_delta_when_the_prior_tree_doubles() {
    let small = delta(256);
    let large = delta(512);
    assert!(
        large as f64 <= small as f64 * 1.5,
        "fresh native admission {small} -> {large}"
    );
}
