use super::*;
use crate::{
    ChargedBytes, ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, KeylessPartition,
    MapKernelFailure, RetainedMapDenial, RetainedRunDenial, RetainedRunOutcome, SerialMemoryBudget,
    SerialRequest,
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};
use worth_proof::CanonicalUniqueVec;

#[test]
fn copied_node_refuses_at_its_own_exact_funding_boundary_before_any_kernel() {
    let memory = SerialMemoryBudget::new(1_000_000);
    let request = SerialRequest::from_memory(memory.clone(), crate::CancellationToken::new(), None);
    let map = ExecutionMap::<_, ()>::from_keyless_partitions(
        (1..=10)
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
            .collect(),
    )
    .unwrap();
    let RetainedRunOutcome::Complete { run: prior, .. } = map
        .run_retained_run(
            ExecutionRequest::serial(&request),
            ExecutionWorkCeiling::new(100),
            |v, context| {
                context.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(*v)
            },
        )
        .unwrap()
    else {
        panic!("native success")
    };
    let held = memory.charged_memory_bytes();
    let scope = ExecutionWorkCeiling::scope_memory_bound_for_test(None, Some(&request)).unwrap();
    let node_bytes = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<super::super::Node<Slot<u64>>>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64;
    // A non-root extremal slot has a copied ancestor during removal. Derive
    // it from this owner's actual deterministic priority, not a magic depth.
    let root = (1..=10)
        .min_by_key(|id| crate::reduction::plan::priority(PartitionIdentity::new(*id)))
        .unwrap();
    let node = PartitionIdentity::new(if root == 1 { 10 } else { 1 });
    let available = scope + prior.additional_charged_bytes() + node_bytes - 1;
    let blocker = memory.reserve(memory.limit() - held - available).unwrap();
    let effects = AtomicUsize::new(0);
    let attempt = prior.advance(
        ExecutionRequest::serial(&request),
        ExecutionWorkCeiling::new(100),
        BTreeMap::from([(
            node,
            KeylessPartition {
                value: 700,
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            },
        )]),
        CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap(),
        |value, context| {
            effects.fetch_add(1, Ordering::SeqCst);
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        },
    );
    assert!(matches!(
        attempt,
        Err(RetainedRunDenial::Native(RetainedMapDenial::Admission(
            LeaseDenial::MemoryExhausted(_)
        )))
    ));
    assert_eq!(effects.load(Ordering::SeqCst), 0);
    assert_eq!(*prior.result(node).unwrap().value(), node.value());
    drop(blocker);
    assert_eq!(
        memory.charged_memory_bytes(),
        held,
        "no partially funded replacement survives"
    );
}
