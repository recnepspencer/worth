use std::sync::atomic::{AtomicUsize, Ordering};

use crate::authority::ExecutionResourceLease;

pub(super) fn run(
    lease: &ExecutionResourceLease<'_>,
    count: usize,
    execute: &(impl Fn(usize) + Sync),
) -> usize {
    let order: Vec<_> = (0..count).collect();
    run_order(lease, &order, execute)
}

pub(super) fn run_order(
    lease: &ExecutionResourceLease<'_>,
    order: &[usize],
    execute: &(impl Fn(usize) + Sync),
) -> usize {
    let width = order
        .len()
        .max(1)
        .min(lease.policy().budget().max_workers().get());
    let additional = (1..width)
        .map_while(|_| lease.try_reserve(1, 0).ok())
        .collect::<Vec<_>>();
    let admitted_workers = additional.len() + 1;
    let next = AtomicUsize::new(0);
    let work = || loop {
        let position = next.fetch_add(1, Ordering::Relaxed);
        if position >= order.len() {
            break;
        }
        execute(order[position]);
    };
    // The caller owns the entry slot. Keep its work on that thread rather
    // than moving it onto another pool thread while the caller waits.
    lease.pool().in_place_scope(|scope| {
        for reservation in additional {
            let work = &work;
            scope.spawn(move |_| {
                let _reservation = reservation;
                work();
            });
        }
        work();
    });
    admitted_workers
}
