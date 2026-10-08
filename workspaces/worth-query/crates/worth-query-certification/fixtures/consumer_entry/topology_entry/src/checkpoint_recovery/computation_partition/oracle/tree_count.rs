//! Independent reducer entries and valid combine pairs for the oracle's f64 reducer.

use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use worth_query_host::facade::primary_graph::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};

static GENERATION: AtomicUsize = AtomicUsize::new(0);
static NODES: AtomicUsize = AtomicUsize::new(0);
thread_local! {
    static PAIR: Cell<(usize, bool)> = const { Cell::new((0, false)) };
}

pub(super) fn reset() {
    GENERATION.fetch_add(1, Ordering::Relaxed);
    NODES.store(0, Ordering::Relaxed);
}

pub(super) fn sum(left: &f64, right: &f64) -> f64 {
    super::COMBINES.fetch_add(1, Ordering::Relaxed);
    let value = left + right;
    // f64 always has valid, admitted eight-byte bits. Each returned second
    // combine completes a node; interrupted first combines complete no node.
    let generation = GENERATION.load(Ordering::Relaxed);
    PAIR.with(|pair| {
        let (previous, pending) = pair.get();
        let pending = previous == generation && pending;
        if pending {
            NODES.fetch_add(1, Ordering::Relaxed);
        }
        pair.set((generation, !pending));
    });
    value
}

pub(super) fn take_nodes() -> usize {
    NODES.swap(0, Ordering::Relaxed)
}

pub(super) fn placement() -> Placement {
    let current = place(Placement::World);
    place(current);
    current
}

/// A declared leaf value faults its reduction independently of placement.
pub(super) const FAULT_LEAF: f64 = -1_000_000.0;
pub(super) fn faulting_sum(left: &f64, right: &f64) -> f64 {
    if *right == FAULT_LEAF {
        super::COMBINES.fetch_add(1, Ordering::Relaxed);
        PAIR.with(|pair| pair.set((GENERATION.load(Ordering::Relaxed), false)));
        panic!("declared reducer leaf fault");
    }
    sum(left, right)
}
