#![forbid(unsafe_code)]

//! Bounded execution of independently owned cases, with canonical result order.

use std::collections::VecDeque;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::Mutex;

/// Execute every case exactly once, including repeated equal case inputs.
/// Cases may not share mutable fate. The compiler requires sendable inputs and
/// results and a shared action; callers reduce returned witnesses after join.
pub fn run<C: Send, R: Send>(cases: Vec<C>, action: impl Fn(C) -> R + Sync) -> Vec<R> {
    run_with_worker_limit(cases, action, None)
}

fn run_with_worker_limit<C: Send, R: Send>(
    cases: Vec<C>,
    action: impl Fn(C) -> R + Sync,
    worker_limit: Option<usize>,
) -> Vec<R> {
    let output = Mutex::new(
        std::iter::repeat_with(|| None)
            .take(cases.len())
            .collect::<Vec<_>>(),
    );
    let jobs = cases
        .into_iter()
        .enumerate()
        .map(|(index, case)| {
            let output = &output;
            let action = &action;
            Box::new(move || {
                let result = action(case);
                output.lock().unwrap()[index] = Some(result);
            }) as Box<dyn FnOnce() + Send + '_>
        })
        .collect();
    run_jobs(jobs, worker_limit);
    output
        .into_inner()
        .unwrap()
        .into_iter()
        .map(|result| result.expect("joined sweep must return every case result"))
        .collect()
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: independent certification harness; source: worth-store-testkit-sweep::run_jobs; one available_parallelism query, one thread::scope and one Scope::spawn site; bounded placement drains every independent case without introducing a leased semantic context"
)]
fn run_jobs(jobs: Vec<Box<dyn FnOnce() + Send + '_>>, worker_limit: Option<usize>) {
    // Match the existing home oracle's maximum worker footprint. Available
    // parallelism controls placement only, never matrix membership or verdicts.
    let workers = worker_limit
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from))
        .clamp(1, 32)
        .min(jobs.len());
    let queue = Mutex::new(jobs.into_iter().enumerate().collect::<VecDeque<_>>());
    let drain = || {
        let mut failures = Vec::new();
        loop {
            let job = queue.lock().unwrap().pop_front();
            let Some((index, job)) = job else {
                break;
            };
            if let Err(failure) = catch_unwind(AssertUnwindSafe(job)) {
                failures.push((index, failure));
            }
        }
        failures
    };
    let mut failures = if workers <= 1 {
        drain()
    } else {
        std::thread::scope(|scope| {
            let handles = (0..workers)
                .map(|_| scope.spawn(&drain))
                .collect::<Vec<_>>();
            let mut failures = Vec::new();
            for handle in handles {
                match handle.join() {
                    Ok(mut worker_failures) => failures.append(&mut worker_failures),
                    Err(failure) => failures.push((usize::MAX, failure)),
                }
            }
            failures
        })
    };
    failures.sort_by_key(|(index, _)| *index);
    if let Some((_, failure)) = failures.into_iter().next() {
        resume_unwind(failure);
    }
}

#[cfg(test)]
mod tests;
