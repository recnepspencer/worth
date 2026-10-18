use super::*;
use crate::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
#[test]
fn funded_group_move_keeps_historical_routes_and_releases_every_grant() {
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 2_000_000, 1_000_000),
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
        .reserve_memory(lease.available_memory_bytes() - room)
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
        .reserve_memory(lease.available_memory_bytes())
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
