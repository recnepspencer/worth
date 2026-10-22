use super::*;
use crate::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, RetainedRunOutcome,
    SerialMemoryBudget, SerialRequest,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_proof::CanonicalUniqueVec;

struct ResultValue {
    value: u64,
    payload: Vec<u8>,
    dropped: Arc<AtomicUsize>,
}
impl Drop for ResultValue {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn native_retained_run_reuses_original_cells_and_only_computes_changed_inputs() {
    let cancellation = CancellationSource::new();
    let request =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let calls = AtomicUsize::new(0);
    let drops = Arc::new(AtomicUsize::new(0));
    let kernel = |input: &u64, context: &mut MapKernelContext<'_, '_>| {
        context.poll()?;
        context.checkpoint(1)?;
        calls.fetch_add(1, Ordering::SeqCst);
        context.poll()?;
        Ok::<_, MapKernelFailure<std::convert::Infallible>>(ResultValue {
            value: *input,
            payload: vec![7; 8192],
            dropped: Arc::clone(&drops),
        })
    };
    let map = ExecutionMap::from_native_partitions(
        (1..=3)
            .map(|value| (PartitionIdentity::new(value), value))
            .collect(),
    )
    .unwrap();
    let RetainedRunOutcome::Complete { run: prior, .. } = map
        .run_retained_run(
            ExecutionRequest::serial(&request),
            ExecutionWorkCeiling::new(None),
            kernel,
        )
        .unwrap()
    else {
        panic!("native retained issue must complete");
    };
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(prior.requires_execution(), None);
    let RetainedRunOutcome::Complete { run: next, .. } = prior
        .advance_native(
            ExecutionRequest::serial(&request),
            ExecutionWorkCeiling::new(None),
            BTreeMap::from([(PartitionIdentity::new(2), 20)]),
            CanonicalUniqueVec::from_btree_set(Default::default()),
            kernel,
        )
        .unwrap()
    else {
        panic!("native retained change must complete");
    };
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    for id in [1, 3].map(PartitionIdentity::new) {
        assert!(prior
            .result(id)
            .unwrap()
            .same_backing(next.result(id).unwrap()));
        assert!(next.result(id).unwrap().native_reuse_available());
    }
    assert_eq!(
        next.result(PartitionIdentity::new(2))
            .unwrap()
            .value()
            .value,
        20
    );
    assert_eq!(
        next.result(PartitionIdentity::new(2))
            .unwrap()
            .value()
            .payload
            .len(),
        8192
    );
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    cancellation.cancel();
    assert!(prior
        .advance_native(
            ExecutionRequest::serial(&request),
            ExecutionWorkCeiling::new(None),
            BTreeMap::<PartitionIdentity, u64>::new(),
            CanonicalUniqueVec::from_btree_set(Default::default()),
            kernel,
        )
        .is_err());
    assert_eq!(
        prior
            .result(PartitionIdentity::new(2))
            .unwrap()
            .value()
            .value,
        2
    );
    drop(prior);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    drop(next);
    assert_eq!(drops.load(Ordering::SeqCst), 4);
}
