use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_operation::{
    application_computation_input_digest, CanonicalEncodingCharge,
};
use worth_query_declaration::facade::application_program::ApplicationComputationInput;

use super::super::WorthQueryManagedComputationResourceDenial;
use super::installed::input_digest;
use super::items::planned_items;
use super::remaining_work::RemainingWork;
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
        .route(PartitionItemId(1), digest(0x0102_0304_0506_0708, 1), |_| {
            Ok(())
        })
        .expect("the first digest owns its partition");
    assert_eq!(first, PartitionIdentity::new(0x0102_0304_0506_0708));
    let (again, _) = routing
        .route(PartitionItemId(2), digest(0x0102_0304_0506_0708, 1), |_| {
            Ok(())
        })
        .expect("an equal digest is the same partition");
    assert_eq!(again, first);
    let (other, _) = routing
        .route(PartitionItemId(3), digest(0x0102_0304_0506_0709, 1), |_| {
            Ok(())
        })
        .expect("another prefix is another partition");
    assert_ne!(other, first);

    let collision = routing
        .route(PartitionItemId(4), digest(0x0102_0304_0506_0708, 2), |_| {
            Ok(())
        })
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
fn an_item_routed_again_leaves_its_last_partition_and_charges_a_fresh_route() {
    let mut routing = ComputationPartitionRouting::default();
    let (left, first) = routing
        .route(PartitionItemId(7), digest(1, 0), |_| Ok(()))
        .expect("the item routes");
    let (joined, again) = routing
        .route(PartitionItemId(7), digest(2, 0), |_| Ok(()))
        .expect("the item routes again");
    assert_ne!(joined, left);
    assert_eq!(again.units(), first.units());
    assert_eq!(routing.partition_of(PartitionItemId(7)), Some(joined));
    assert_eq!(routing.partitions().collect::<Vec<_>>(), [joined]);
}

#[test]
fn a_plan_naming_items_twice_is_denied_at_the_least() {
    let entries = [9, 3, 9, 3].map(|item| (PartitionItemId(item), ()));
    assert_eq!(
        planned_items::<(), ()>(entries.into()).err(),
        Some(WorthQueryPartitionedComputationDenial::DuplicateItem {
            item: PartitionItemId(3)
        })
    );
}

struct Words;
impl ApplicationComputationInput for Words {
    type Value = std::collections::BTreeMap<String, u64>;
    const IDENTITY: &'static str = "worth.query.tests.encoding-meter-words.v1";
}

/// The input digest sums its scratch growths as a partition key does: a
/// ceiling every single growth fits but their sum passes refuses the input.
/// A map buffers its entries, so its encoding grows scratch repeatedly.
#[test]
fn an_input_whose_scratch_growths_each_fit_but_whose_sum_does_not_is_refused() {
    let input = (0..64)
        .map(|word| (format!("word-{word:04}"), word))
        .collect();
    let mut growths = Vec::new();
    application_computation_input_digest::<Words, _, _>(&input, &mut |charge| {
        if let CanonicalEncodingCharge::Scratch(bytes) = charge {
            growths.push(bytes);
        }
        Ok::<_, ()>(())
    })
    .unwrap();
    let largest = *growths.iter().max().unwrap();
    let total = growths.iter().sum::<u64>();
    assert!(
        largest < total,
        "the encoding grows more than once: {growths:?}"
    );

    let mut work = RemainingWork::declared(u64::MAX);
    assert_eq!(
        input_digest::<Words, ()>(&input, &mut work, total - 1),
        Err(WorthQueryPartitionedComputationDenial::Resource(
            WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted
        ))
    );
    let mut work = RemainingWork::declared(u64::MAX);
    assert!(input_digest::<Words, ()>(&input, &mut work, total).is_ok());
}
