//! Real retained grants guard membership path copying, separately from routes.
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionAllocationPolicy, ExecutionResourceLease, KeyedItem,
    KeyedPartitioner, LeaseRequest, PartitionItemId,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};
fn lease() -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}
fn entry(item: u64, key: u64) -> KeyedItem<u64> {
    KeyedItem {
        item: PartitionItemId(item),
        key,
        partition: PartitionIdentity::new(key),
    }
}
fn edit_bytes(count: u64) -> u64 {
    let original = lease();
    let fresh = lease();
    let mut routing = KeyedPartitioner::new();
    for id in 0..count {
        routing
            .upsert_in(
                &entry(id, 1),
                ExecutionAllocationPolicy::Execution(&original),
            )
            .unwrap();
    }
    let predecessor = routing.clone();
    routing
        .remove_in(
            PartitionItemId(count - 1),
            ExecutionAllocationPolicy::Execution(&fresh),
        )
        .unwrap();
    assert_eq!(
        routing.members(PartitionIdentity::new(1)).unwrap().len(),
        count as usize - 1
    );
    assert_eq!(
        predecessor
            .members(PartitionIdentity::new(1))
            .unwrap()
            .len(),
        count as usize
    );
    let bytes = fresh.charged_memory_bytes();
    drop(routing);
    assert_eq!(fresh.charged_memory_bytes(), 0);
    drop(predecessor);
    assert_eq!(original.charged_memory_bytes(), 0);
    bytes
}
#[test]
fn membership_edit_admits_a_path_when_unchanged_group_doubles() {
    let small = edit_bytes(256);
    let large = edit_bytes(512);
    assert!(
        large as f64 <= small as f64 * 1.5,
        "membership delta bytes {small} -> {large}"
    );
}
#[test]
fn persistent_members_equal_independent_sets_through_rekeys_and_retained_predecessors() {
    let original = lease();
    let policy = ExecutionAllocationPolicy::Execution(&original);
    let mut routing = KeyedPartitioner::new();
    let mut truth = BTreeMap::<u64, u64>::new();
    let mut random = 1u64;
    for step in 0..1024 {
        let prior = routing.clone();
        let prior_truth = truth.clone();
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let id = (random >> 32) % 64;
        let key = (random >> 48) % 8;
        if random.is_multiple_of(4) {
            routing.remove_in(PartitionItemId(id), policy).unwrap();
            truth.remove(&id);
        } else {
            routing.upsert_in(&entry(id, key), policy).unwrap();
            truth.insert(id, key);
        }
        for (candidate, expected) in [(&routing, &truth), (&prior, &prior_truth)] {
            for key in 0..8 {
                let expected = expected
                    .iter()
                    .filter_map(|(&id, &group)| (group == key).then_some(PartitionItemId(id)))
                    .collect::<BTreeSet<_>>();
                let found = candidate
                    .members(PartitionIdentity::new(key))
                    .map(|v| v.iter().copied().collect::<BTreeSet<_>>())
                    .unwrap_or_default();
                assert_eq!(found, expected, "step={step} key={key}");
            }
        }
    }
    drop(routing);
    assert_eq!(original.charged_memory_bytes(), 0);
}
