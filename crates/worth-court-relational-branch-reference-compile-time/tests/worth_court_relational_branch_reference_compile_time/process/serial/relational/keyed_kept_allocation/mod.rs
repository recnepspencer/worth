// A Native consumer measures backend admission with its permitted allocator probe.
#[allow(unused_imports)] use crate::keyed_kept_allocation::isolated_context::*;
use {stats_alloc::Region, stats_alloc::StatsAlloc, stats_alloc::INSTRUMENTED_SYSTEM};
use {worth_execution::KeyedEditDenial, worth_execution::KeyedItem, worth_execution::KeyedPartitioner, worth_execution::PartitionItemId};
use {worth_foundational::PartitionIdentity};

#[test]
fn a_refused_keyed_subset_allocates_nothing_before_admission() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    // The isolated child has no sibling tests allocating inside this region.
    let mut keyed = worth_execution::KeyedPartitioner::new();
    for item in 0..32_u64 {
        keyed
            .upsert(
                KeyedItem {
                    item: PartitionItemId(item),
                    key: item % 4,
                    partition: worth_foundational::PartitionIdentity::new(item % 4),
                },
                |_| Ok::<(), ()>(()),
            )
            .unwrap();
    }
    let allocations = stats_alloc::Region::new(ALLOCATOR);
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
    assert!(matches!(refused, Err(worth_execution::KeyedEditDenial::Admission(_))));
}
