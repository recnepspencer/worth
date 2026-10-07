//! A Native consumer measures backend admission with its permitted allocator probe.
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use worth_execution::{KeyedEditDenial, KeyedItem, KeyedPartitioner, PartitionItemId};
use worth_foundational::PartitionIdentity;

#[global_allocator]
static ALLOCATOR: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

#[test]
fn a_refused_keyed_subset_allocates_nothing_before_admission() {
    // A dedicated binary has no sibling tests allocating inside this region.
    let mut keyed = KeyedPartitioner::new();
    for item in 0..32_u64 {
        keyed
            .upsert(
                KeyedItem {
                    item: PartitionItemId(item),
                    key: item % 4,
                    partition: PartitionIdentity::new(item % 4),
                },
                |_| Ok::<(), ()>(()),
            )
            .unwrap();
    }
    let allocations = Region::new(ALLOCATOR);
    let refused = keyed.kept(
        |_| true,
        |bound| {
            let measured = allocations.change();
            assert_eq!(
                measured.allocations + measured.reallocations,
                0,
                "no proportional allocation may precede admission"
            );
            Err(bound)
        },
    );
    assert!(matches!(refused, Err(KeyedEditDenial::Admission(_))));
}
