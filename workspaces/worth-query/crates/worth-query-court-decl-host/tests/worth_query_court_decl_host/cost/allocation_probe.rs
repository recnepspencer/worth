//! The allocation probe counts only what the measuring thread allocates inside
//! the measured closure, exactly, so a 10× court reads a 10× ratio.
use std::hint::black_box;
use worth_testkit_foundational::allocation_probe::{AllocationCount, AllocationProbe};

fn boxes(n: u64) -> AllocationCount {
    // The vector is allocated before, and the boxes freed after, the closure.
    let mut held = Vec::with_capacity(n as usize);
    let ((), count) = AllocationProbe::measure(|| held.extend((0..n).map(Box::new)));
    drop(black_box(held));
    count
}

#[test]
fn ten_times_the_boxes_count_exactly_ten_times() {
    let one = boxes(100);
    assert_eq!(
        one,
        AllocationCount {
            allocations: 100,
            bytes: 800
        }
    );
    assert_eq!(
        boxes(1_000),
        AllocationCount {
            allocations: 1_000,
            bytes: 8_000
        }
    );
}

#[test]
fn another_threads_allocations_are_not_counted() {
    let spawned = |n: u64| {
        AllocationProbe::measure(|| {
            std::thread::scope(|scope| {
                scope.spawn(|| (0..n).for_each(|value| drop(black_box(Box::new(value)))));
            })
        })
        .1
    };
    assert_eq!(spawned(1), spawned(1_000));
}
