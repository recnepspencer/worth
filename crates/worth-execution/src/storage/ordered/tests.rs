use super::*;
use crate::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
#[test]
fn ranks_follow_native_edits_without_changing_historical_order_or_funding() {
    let lease = lease(2_000_000);
    let policy = ExecutionAllocationPolicy::Execution(&lease);
    let mut map = ExecutionOrderedMap::default();
    for key in [9u64, 1, 7, 3, 5] {
        map.insert_cloned_in(&key, &key, policy).unwrap();
    }
    let historical = map.clone();
    map.remove_in(&3, policy).unwrap();
    map.insert_cloned_in(&4, &40, policy).unwrap();
    let before = lease.charged_memory_bytes();
    for (rank, key) in [1u64, 4, 5, 7, 9].into_iter().enumerate() {
        assert_eq!(map.rank_of(&key), Some(rank));
        assert_eq!(map.entry_at(rank).map(|(key, _)| *key), Some(key));
    }
    assert_eq!(map.rank_of(&3), None);
    assert_eq!(map.entry_at(5), None);
    assert_eq!(historical.entry_at(1), Some((&3, &3)));
    assert_eq!(lease.charged_memory_bytes(), before);
    drop(map);
    drop(historical);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
fn lease(bytes: u64) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, bytes, 1_000_000),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}
#[test]
fn shared_roots_keep_direct_grants_and_only_new_paths_need_room() {
    let lease = lease(2_000_000);
    let policy = ExecutionAllocationPolicy::Execution(&lease);
    let mut map = ExecutionOrderedMap::default();
    for key in 0u64..128 {
        map.insert_cloned_in(&key, &format!("value-{key}"), policy)
            .unwrap();
    }
    let held = lease.charged_memory_bytes();
    assert!(held > 0);
    let historical = map.clone();
    assert_eq!(
        lease.charged_memory_bytes(),
        held,
        "sharing a root allocates nothing"
    );
    // Leave less room than one complete retained map, while a real changed
    // path still fits. The actual lease, not a descriptor callback, decides.
    let room = map.retained_bytes() / 2;
    let blocker = lease
        .reserve_memory(lease.available_memory_bytes() - room)
        .unwrap();
    let grown = "changed".repeat(512);
    map.insert_cloned_in(&70, &grown, policy).unwrap();
    assert_eq!(historical.get(&70).map(String::as_str), Some("value-70"));
    assert_eq!(map.get(&70).map(String::as_str), Some(grown.as_str()));
    drop(blocker);
    drop(historical);
    let one_view = lease.charged_memory_bytes();
    assert_eq!(
        one_view,
        map.retained_bytes(),
        "no root or grant history remains"
    );
    drop(map);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
#[test]
fn growth_refuses_before_copy_and_keeps_the_original_root() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    struct Owned {
        value: String,
        copies: Arc<AtomicUsize>,
    }
    impl Clone for Owned {
        fn clone(&self) -> Self {
            self.copies.fetch_add(1, Ordering::Relaxed);
            Self {
                value: self.value.clone(),
                copies: Arc::clone(&self.copies),
            }
        }
    }
    impl crate::ChargedBytes for Owned {
        fn additional_charged_bytes(&self) -> u64 {
            self.value.capacity() as u64
        }
    }
    let lease = lease(8192);
    let policy = ExecutionAllocationPolicy::Execution(&lease);
    let copies = Arc::new(AtomicUsize::new(0));
    let mut map = ExecutionOrderedMap::default();
    map.insert_cloned_in(
        &1u64,
        &Owned {
            value: "small".to_owned(),
            copies: Arc::clone(&copies),
        },
        policy,
    )
    .unwrap();
    let root = map.root.clone();
    let held = lease.charged_memory_bytes();
    copies.store(0, Ordering::Relaxed);
    assert!(map
        .insert_cloned_in(
            &1,
            &Owned {
                value: "x".repeat(1024 * 1024),
                copies: Arc::clone(&copies)
            },
            policy
        )
        .is_err());
    assert_eq!(
        copies.load(Ordering::Relaxed),
        0,
        "the denied heap value was never copied"
    );
    assert!(Arc::ptr_eq(
        root.as_ref().unwrap(),
        map.root.as_ref().unwrap()
    ));
    assert_eq!(map.get(&1).unwrap().value, "small");
    assert_eq!(lease.charged_memory_bytes(), held);
    drop(root);
    drop(map);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
#[test]
fn rotations_and_removals_match_independent_ordered_truth_with_old_views() {
    let lease = lease(2_000_000);
    let policy = ExecutionAllocationPolicy::Execution(&lease);
    let mut map = ExecutionOrderedMap::default();
    let mut truth = std::collections::BTreeMap::new();
    for key in (0u64..80).map(|i| (i * 37) % 80) {
        map.insert_cloned_in(&key, &(key + 100), policy).unwrap();
        truth.insert(key, key + 100);
    }
    let original = map.clone();
    for key in (0u64..80).step_by(3) {
        assert!(map.remove_in(&key, policy).unwrap());
        truth.remove(&key);
        assert!(map.iter().eq(truth.iter()));
        assert_eq!(map.len(), truth.len());
    }
    assert_eq!(original.len(), 80);
    assert!(original.root.as_ref().unwrap().height < 2 * usize::BITS as u8);
    drop(map);
    drop(original);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
