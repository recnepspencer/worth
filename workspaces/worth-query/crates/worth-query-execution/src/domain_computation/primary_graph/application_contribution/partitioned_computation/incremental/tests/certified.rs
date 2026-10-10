//! A test places its requests' managed computations: serially, on a lease,
//! or on a lease whose full reductions are certified. Every placement reduces
//! to the same result and charges the same work, and only a certified one
//! reduces again, through the serial oracle and the perturbed backend.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::super::super::request_execution::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
use super::*;

/// The combines of this file's one test, on whichever thread they ran.
static COMBINES: AtomicUsize = AtomicUsize::new(0);

fn counting_sum(left: &u64, right: &u64) -> u64 {
    COMBINES.fetch_add(1, Ordering::Relaxed);
    left + right
}

#[test]
fn every_placement_agrees_and_only_a_certified_one_reduces_again() {
    let world = installed_authorization_world(true);
    let installed = installed_over(4, StatusRead::Gather(1), counting_sum);
    let placed = |placement| {
        let before = place(placement);
        COMBINES.store(0, Ordering::Relaxed);
        let attempt = attempt(&world, &installed, None);
        place(before);
        (attempt.outcome, COMBINES.load(Ordering::Relaxed))
    };
    let two = NonZeroUsize::new(2).unwrap();

    let serial = placed(Placement::Serial);
    assert_eq!(serial.0.as_ref().unwrap().0, 1 + 2 + 3 + 4);
    assert_eq!(serial.1, 8, "four nodes take two combines each");
    assert_eq!(placed(Placement::Leased(two)), serial);
    assert_eq!(
        placed(Placement::Certified {
            workers: two,
            seed: 0x5eed,
        }),
        (serial.0, 3 * 8),
        "the run, the oracle and the perturbed backend each reduce once"
    );
}
