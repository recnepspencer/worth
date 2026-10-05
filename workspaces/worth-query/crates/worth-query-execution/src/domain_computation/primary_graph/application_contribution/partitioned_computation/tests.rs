use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;

use super::routing::{ComputationPartitionRouting, ComputationPartitionRoutingDenial};
use super::WorthQueryPartitionedComputationDenial;

fn digest(prefix: u64, tail: u8) -> [u8; 32] {
    let mut digest = [0_u8; 32];
    digest[..8].copy_from_slice(&prefix.to_be_bytes());
    digest[31] = tail;
    digest
}

#[test]
fn two_digests_sharing_their_first_eight_bytes_are_a_collision_naming_the_partition() {
    let mut routing = ComputationPartitionRouting::default();
    let (first, _) = routing
        .route(PartitionItemId(1), digest(0x0102_0304_0506_0708, 1))
        .expect("the first digest owns its partition");
    assert_eq!(first, PartitionIdentity::new(0x0102_0304_0506_0708));
    let (again, _) = routing
        .route(PartitionItemId(2), digest(0x0102_0304_0506_0708, 1))
        .expect("an equal digest is the same partition");
    assert_eq!(again, first);
    let (other, _) = routing
        .route(PartitionItemId(3), digest(0x0102_0304_0506_0709, 1))
        .expect("another prefix is another partition");
    assert_ne!(other, first);

    let collision = routing
        .route(PartitionItemId(4), digest(0x0102_0304_0506_0708, 2))
        .expect_err("a different digest may not share the partition identity");
    assert_eq!(
        collision,
        ComputationPartitionRoutingDenial::IdentityCollision(first)
    );
    assert_eq!(
        WorthQueryPartitionedComputationDenial::<()>::from(collision),
        WorthQueryPartitionedComputationDenial::PartitionIdentityCollision { partition: first }
    );
    // The refused item was not routed, and the partition keeps its members.
    assert_eq!(
        routing.members(first).collect::<Vec<_>>(),
        [PartitionItemId(1), PartitionItemId(2)]
    );
}

#[test]
fn an_item_identity_named_twice_is_denied() {
    let mut routing = ComputationPartitionRouting::default();
    routing
        .route(PartitionItemId(7), digest(1, 0))
        .expect("the item routes once");
    let duplicate = routing
        .route(PartitionItemId(7), digest(2, 0))
        .expect_err("the plan may not name one item twice");
    assert_eq!(
        WorthQueryPartitionedComputationDenial::<()>::from(duplicate),
        WorthQueryPartitionedComputationDenial::DuplicateItem {
            item: PartitionItemId(7)
        }
    );
}
