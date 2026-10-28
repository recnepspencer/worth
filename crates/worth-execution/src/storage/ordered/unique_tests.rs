use super::*;
use crate::{CancellationSource, ExecutionResourceLease, LeaseRequest};
use std::{collections::BTreeMap, num::NonZeroUsize};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(cancellation: &CancellationSource) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(2_000_000), Some(1_000_000)),
        ),
        cancellation: cancellation.token(),
        deadline: None,
    })
    .unwrap()
}

fn structure<K: Ord, V>(link: &Link<K, V>) -> (u8, usize) {
    let Some(node) = link else { return (0, 0) };
    assert_eq!(Arc::strong_count(node), 1, "no root or child escapes");
    let (left, lc) = structure(&node.left);
    let (right, rc) = structure(&node.right);
    assert!(left.abs_diff(right) <= 1);
    assert_eq!(node.height, left.max(right) + 1);
    assert_eq!(node.count, lc + rc + 1);
    (node.height, node.count)
}

#[test]
fn unique_removals_keep_order_balance_and_original_cell_funding() {
    let cancellation = CancellationSource::new();
    let lease = lease(&cancellation);
    let request = ExecutionRequest::leased(&lease);
    let mut map = ExecutionUniqueOrderedMap::default();
    let mut truth = BTreeMap::new();
    for key in (0u64..127).map(|i| (i * 37) % 127) {
        map.insert_cloned_native_for_request(&key, &(key + 1000), request)
            .unwrap();
        truth.insert(key, key + 1000);
        structure(&map.map.root);
    }
    assert_eq!(map.accounted_bytes(), Some(lease.charged_memory_bytes()));
    for key in (0u64..127).map(|i| (i * 53) % 127) {
        assert!(map.remove(&key));
        truth.remove(&key);
        assert!(map.iter().eq(truth.iter()));
        assert_eq!(map.len(), truth.len());
        structure(&map.map.root);
        assert_eq!(map.accounted_bytes(), Some(lease.charged_memory_bytes()));
    }
    assert!(!map.remove(&0));
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn unique_rollback_after_real_stop_needs_no_room_or_payload_clone() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Payload(Arc<AtomicUsize>);
    impl Clone for Payload {
        fn clone(&self) -> Self {
            self.0.fetch_add(1, Ordering::Relaxed);
            Self(Arc::clone(&self.0))
        }
    }
    let cancellation = CancellationSource::new();
    let lease = lease(&cancellation);
    let request = ExecutionRequest::leased(&lease);
    let copies = Arc::new(AtomicUsize::new(0));
    let value = Payload(Arc::clone(&copies));
    let mut map = ExecutionUniqueOrderedMap::default();
    for key in [4u64, 2, 6, 1, 3, 5, 7] {
        map.insert_cloned_native_for_request(&key, &value, request)
            .unwrap();
    }
    copies.store(0, Ordering::Relaxed);
    let held = lease.charged_memory_bytes();
    let blocker = lease
        .reserve_memory(lease.available_memory_bytes().unwrap())
        .unwrap();
    assert!(map
        .insert_cloned_native_for_request(&8, &value, request)
        .is_err());
    assert_eq!(copies.load(Ordering::Relaxed), 0);
    assert_eq!(map.len(), 7);
    structure(&map.map.root);
    cancellation.cancel();
    assert!(
        request.consult().is_err(),
        "the original payer really stopped"
    );
    for key in [4, 2, 6, 1, 3, 5, 7] {
        assert!(map.remove(&key));
    }
    assert_eq!(copies.load(Ordering::Relaxed), 0);
    assert!(map.is_empty());
    assert_eq!(lease.charged_memory_bytes(), 2_000_000 - held);
    drop(blocker);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
