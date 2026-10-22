use super::*;
use crate::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn worker_only_rekey_skips_heap_census_and_accounted_transition_is_explicit() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CENSUS: AtomicUsize = AtomicUsize::new(0);
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Key(u64);
    impl ChargedBytes for Key {
        fn additional_charged_bytes(&self) -> u64 {
            CENSUS.fetch_add(1, Ordering::SeqCst);
            16
        }
    }
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let policy = ExecutionAllocationPolicy::Execution(&lease);
    let mut routing = KeyedPartitioner::new();
    let entry = |item, key| KeyedItem {
        item: PartitionItemId(item),
        key: Key(key),
        partition: PartitionIdentity::new(key),
    };
    routing.upsert_in(&entry(1, 1), policy).unwrap();
    routing.upsert_in(&entry(2, 1), policy).unwrap();
    let original = routing.clone();
    routing.upsert_in(&entry(1, 2), policy).unwrap();
    routing.remove_in(PartitionItemId(2), policy).unwrap();
    assert_eq!(
        routing.route(PartitionItemId(1)),
        Some(PartitionIdentity::new(2))
    );
    assert_eq!(
        original.route(PartitionItemId(1)),
        Some(PartitionIdentity::new(1))
    );
    assert_eq!(routing.charged_bytes(), None);
    assert_eq!(routing.items.accounted_bytes(), None);
    assert_eq!(CENSUS.load(Ordering::SeqCst), 0);
    assert_eq!(lease.charged_memory_bytes(), 0);
    let accounted = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    routing
        .upsert_in(
            &entry(3, 3),
            ExecutionAllocationPolicy::Execution(&accounted),
        )
        .unwrap();
    assert!(CENSUS.load(Ordering::SeqCst) > 0);
    assert!(routing.charged_bytes().is_some());
    assert_eq!(
        routing.route(PartitionItemId(3)),
        Some(PartitionIdentity::new(3))
    );
    assert_eq!(
        original.route(PartitionItemId(2)),
        Some(PartitionIdentity::new(1))
    );
    drop(routing);
    assert_eq!(accounted.charged_memory_bytes(), 0);
}
#[test]
fn funded_group_move_keeps_historical_routes_and_releases_every_grant() {
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(2_000_000), Some(1_000_000)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let policy = ExecutionAllocationPolicy::Execution(&lease);
    let mut routing = KeyedPartitioner::new();
    for id in 0..128 {
        routing
            .upsert_in(
                &KeyedItem {
                    item: PartitionItemId(id),
                    key: id / 2,
                    partition: PartitionIdentity::new(id / 2),
                },
                policy,
            )
            .unwrap();
    }
    let held = lease.charged_memory_bytes();
    let historical = routing.clone();
    assert_eq!(lease.charged_memory_bytes(), held);
    let room = routing.charged_bytes().unwrap() / 3;
    let blocker = lease
        .reserve_memory(
            lease
                .available_memory_bytes()
                .expect("fixture installs finite memory")
                - room,
        )
        .unwrap();
    routing
        .upsert_in(
            &KeyedItem {
                item: PartitionItemId(0),
                key: 1,
                partition: PartitionIdentity::new(1),
            },
            policy,
        )
        .unwrap();
    assert_eq!(
        routing.route(PartitionItemId(0)),
        Some(PartitionIdentity::new(1))
    );
    assert_eq!(
        historical.route(PartitionItemId(0)),
        Some(PartitionIdentity::new(0))
    );
    assert_eq!(
        routing.members(PartitionIdentity::new(0)).unwrap(),
        &BTreeSet::from([PartitionItemId(1)])
    );
    let before = lease.charged_memory_bytes();
    assert!(matches!(
        routing.upsert_in(
            &KeyedItem {
                item: PartitionItemId(4),
                key: 99,
                partition: PartitionIdentity::new(1)
            },
            policy
        ),
        Err(KeyedEditDenial::Keyed(
            KeyedDenial::IdentityCollision { .. }
        ))
    ));
    assert_eq!(lease.charged_memory_bytes(), before);
    assert!(matches!(routing.upsert_in(&KeyedItem {
        item:PartitionItemId(4),key:1,partition:PartitionIdentity::new(2),
    },policy),Err(KeyedEditDenial::Keyed(KeyedDenial::KeyIdentityChanged {partition})) if partition==PartitionIdentity::new(1)));
    assert_eq!(lease.charged_memory_bytes(), before);
    drop(blocker);
    let blocker = lease
        .reserve_memory(
            lease
                .available_memory_bytes()
                .expect("fixture installs finite memory"),
        )
        .unwrap();
    assert!(routing
        .upsert_in(
            &KeyedItem {
                item: PartitionItemId(0),
                key: 2,
                partition: PartitionIdentity::new(2)
            },
            policy
        )
        .is_err());
    assert_eq!(
        routing.route(PartitionItemId(0)),
        Some(PartitionIdentity::new(1))
    );
    drop(blocker);
    drop(routing);
    assert!(
        lease.charged_memory_bytes() > 0,
        "old root still owns its cells"
    );
    drop(historical);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
