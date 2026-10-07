use crate::authority::ExecutionResourceLease;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

/// Borrowed dispatch acquires items by atomic index into a scheduling-order buffer.
pub(super) fn run_order(
    lease: &ExecutionResourceLease<'_>,
    order: &[usize],
    execute: &(impl Fn(usize) + Sync),
) -> usize {
    let next = AtomicUsize::new(0);
    let work = || loop {
        let position = next.fetch_add(1, Ordering::Relaxed);
        if position >= order.len() {
            break;
        }
        execute(order[position]);
    };
    run_workers(lease, order.len(), &work)
}

/// Only owned dispatch locks a queue to move an input into exactly one worker.
pub(super) fn run_tasks<I: Iterator + Send>(
    lease: &ExecutionResourceLease<'_>,
    count: usize,
    tasks: I,
    execute: &(impl Fn(I::Item) + Sync),
) -> usize {
    let tasks = Mutex::new(tasks);
    let work = || loop {
        let task = tasks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .next();
        let Some(task) = task else {
            break;
        };
        execute(task);
    };
    run_workers(lease, count, &work)
}

/// Worker admission and joining are shared by both acquisition mechanisms.
fn run_workers(
    lease: &ExecutionResourceLease<'_>,
    count: usize,
    work: &(impl Fn() + Sync),
) -> usize {
    let width = count
        .max(1)
        .min(lease.policy().budget().max_workers().get());
    let additional = (1..width)
        .map_while(|_| lease.try_reserve(1, 0).ok())
        .collect::<Vec<_>>();
    let admitted_workers = additional.len() + 1;
    // The caller's entry executes on its own thread.
    lease.pool().in_place_scope(|scope| {
        for reservation in additional {
            scope.spawn(move |_| {
                let _reservation = reservation;
                work();
            });
        }
        work();
    });
    admitted_workers
}
