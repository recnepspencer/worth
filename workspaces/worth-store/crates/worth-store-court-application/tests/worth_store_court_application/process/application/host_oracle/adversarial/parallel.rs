//! Parallel worker execution helpers for C4b adversarial barrier sweeps and
//! orthogonal matrices, failing fast on unexpected panics.
use super::{media_fault_sweep::panic_message, BarrierKill, TAIL_VARIANCE_BOUND};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Run every measured barrier ordinal `1..=total` in parallel across isolated
/// per-worker fixtures, failing fast on the first unexpected panic.
pub(super) fn run_parallel_barriers<W, T>(
    label: &str,
    total: u64,
    prepare_worker: impl Fn() -> W + Sync,
    run_case: impl Fn(&W, u64) -> (BarrierKill, T) + Sync,
) -> Vec<(u64, BarrierKill, T)>
where
    T: Send,
{
    assert!(
        total > 0,
        "{label}: target operation must execute durable syncs"
    );
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(1, 8)
        .min(total as usize);
    let next = AtomicU64::new(1);
    let aborted = Arc::new(AtomicBool::new(false));
    let results: Mutex<Vec<Option<Result<(BarrierKill, T), String>>>> =
        Mutex::new((0..total as usize).map(|_| None).collect());
    let (next_ref, results_ref) = (&next, &results);
    let (prep_ref, run_ref) = (&prepare_worker, &run_case);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            let aborted = Arc::clone(&aborted);
            scope.spawn(move || {
                let worker_state = prep_ref();
                loop {
                    if aborted.load(Ordering::Acquire) {
                        break;
                    }
                    let ordinal = next_ref.fetch_add(1, Ordering::Relaxed);
                    if ordinal > total {
                        break;
                    }
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        run_ref(&worker_state, ordinal)
                    }))
                    .map_err(|payload| panic_message(&*payload));
                    if outcome.is_err() {
                        aborted.store(true, Ordering::Release);
                    }
                    results_ref.lock().unwrap()[(ordinal - 1) as usize] = Some(outcome);
                }
            });
        }
    });
    collect_barrier_results(label, total, results.into_inner().unwrap())
}

/// Prepare per-worker calibrated fixtures `(W, worker_total)` and run every
/// ordinal `1..=max(worker_total)` only on workers whose calibrated total is
/// `>= ordinal`, requiring an exact parked OS kill at EVERY SINGLE barrier.
pub(super) fn run_parallel_calibrated_barriers<W: Send, T: Send>(
    label: &str,
    prepare_worker: impl Fn() -> (W, u64) + Sync,
    run_case: impl Fn(&W, u64, u64) -> T + Sync,
) -> Vec<(u64, BarrierKill, T)> {
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(1, 6);
    let prepared: Vec<(W, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers).map(|_| scope.spawn(&prepare_worker)).collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let max_total = prepared.iter().map(|(_, t)| *t).max().unwrap_or(0);
    assert!(
        max_total > 0,
        "{label}: target operation must execute durable syncs"
    );
    let queue = Mutex::new((1..=max_total).collect::<Vec<u64>>());
    let aborted = Arc::new(AtomicBool::new(false));
    let results: Mutex<Vec<Option<Result<T, String>>>> =
        Mutex::new((0..max_total as usize).map(|_| None).collect());
    std::thread::scope(|scope| {
        for (w, w_total) in prepared {
            let (queue, results, run_ref) = (&queue, &results, &run_case);
            let aborted = Arc::clone(&aborted);
            scope.spawn(move || loop {
                if aborted.load(Ordering::Acquire) {
                    break;
                }
                let ordinal = {
                    let mut q = queue.lock().unwrap();
                    let Some(pos) = q.iter().position(|&o| o <= w_total) else {
                        break;
                    };
                    q.swap_remove(pos)
                };
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run_ref(&w, ordinal, w_total)
                }))
                .map_err(|payload| panic_message(&*payload));
                if outcome.is_err() {
                    aborted.store(true, Ordering::Release);
                }
                results.lock().unwrap()[(ordinal - 1) as usize] = Some(outcome);
            });
        }
    });
    let slots = results.into_inner().unwrap();
    if let Some((idx, msg)) = slots.iter().enumerate().find_map(|(i, s)| match s {
        Some(Err(m)) => Some((i, m)),
        _ => None,
    }) {
        panic!("{label} sync#{} of {max_total} failed: {msg}", idx + 1);
    }
    let mut out = Vec::with_capacity(max_total as usize);
    for (idx, slot) in slots.into_iter().enumerate() {
        let ordinal = (idx as u64) + 1;
        match slot {
            Some(Ok(val)) => out.push((ordinal, BarrierKill::Killed, val)),
            Some(Err(msg)) => panic!("{label} sync#{ordinal} of {max_total} failed: {msg}"),
            None => panic!("{label} sync#{ordinal} of {max_total} skipped after earlier failure"),
        }
    }
    out
}

fn collect_barrier_results<T>(
    label: &str,
    total: u64,
    slots: Vec<Option<Result<(BarrierKill, T), String>>>,
) -> Vec<(u64, BarrierKill, T)> {
    if let Some((idx, msg)) = slots.iter().enumerate().find_map(|(i, s)| match s {
        Some(Err(m)) => Some((i, m)),
        _ => None,
    }) {
        panic!("{label} sync#{} of {total} failed: {msg}", idx + 1);
    }
    let mut out = Vec::with_capacity(total as usize);
    let mut killed = 0u64;
    let mut min_tail = total;
    for (idx, slot) in slots.into_iter().enumerate() {
        let ordinal = (idx as u64) + 1;
        match slot {
            Some(Ok((kill, val))) => {
                match kill {
                    BarrierKill::Killed => killed += 1,
                    BarrierKill::CompletedAtTail { actual } => min_tail = min_tail.min(actual),
                }
                out.push((ordinal, kill, val));
            }
            Some(Err(msg)) => panic!("{label} sync#{ordinal} of {total} failed: {msg}"),
            None => panic!("{label} sync#{ordinal} of {total} skipped after earlier failure"),
        }
    }
    assert!(
        killed >= min_tail && killed >= total.saturating_sub(TAIL_VARIANCE_BOUND),
        "{label}: expected at least {} killed barriers (min_tail={min_tail}, total={total}), got {killed}",
        min_tail.max(total.saturating_sub(TAIL_VARIANCE_BOUND))
    );
    out
}

/// Run independent adversarial matrix items across up to 6 worker threads,
/// failing fast on the first panic and reporting all captured failures.
pub(super) fn run_parallel_items<T: Sync>(suite: &str, items: &[T], run: impl Fn(&T) + Sync) {
    let next = AtomicUsize::new(0);
    let aborted = Arc::new(AtomicBool::new(false));
    let errors = Mutex::new(Vec::<String>::new());
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(1, 6);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            let (next, errors, run) = (&next, &errors, &run);
            let aborted = Arc::clone(&aborted);
            scope.spawn(move || loop {
                if aborted.load(Ordering::Acquire) {
                    break;
                }
                let idx = next.fetch_add(1, Ordering::Relaxed);
                if idx >= items.len() {
                    break;
                }
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run(&items[idx]);
                }));
                if let Err(payload) = res {
                    aborted.store(true, Ordering::Release);
                    errors
                        .lock()
                        .unwrap()
                        .push(format!("item#{idx}: {}", panic_message(&*payload)));
                }
            });
        }
    });
    let errs = errors.into_inner().unwrap();
    assert!(errs.is_empty(), "{suite} failures:\n{}", errs.join("\n"));
}

/// Run independent indexed cases across pre-built per-worker states, failing
/// fast on the first panic via `Arc<AtomicBool>` and reporting the first real
/// failure before any skipped slots.
pub(super) fn run_parallel_worker_cases<W: Sync, C: std::fmt::Debug + Sync, T: Send>(
    label: &str,
    workers: &[W],
    cases: &[C],
    run_case: impl Fn(&W, &C) -> T + Sync,
) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let aborted = Arc::new(AtomicBool::new(false));
    let results: Mutex<Vec<Option<Result<T, String>>>> =
        Mutex::new((0..cases.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for worker in workers {
            let (next, results, run_case) = (&next, &results, &run_case);
            let aborted = Arc::clone(&aborted);
            scope.spawn(move || loop {
                if aborted.load(Ordering::Acquire) {
                    break;
                }
                let idx = next.fetch_add(1, Ordering::Relaxed);
                let Some(case) = cases.get(idx) else {
                    break;
                };
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run_case(worker, case)
                }))
                .map_err(|payload| panic_message(&*payload));
                if outcome.is_err() {
                    aborted.store(true, Ordering::Release);
                }
                results.lock().unwrap()[idx] = Some(outcome);
            });
        }
    });
    collect_case_results(label, cases, results.into_inner().unwrap())
}

/// Collect fail-fast parallel case outcomes, surfacing the first real error
/// before any skipped (`None`) slots.
pub(super) fn collect_case_results<C: std::fmt::Debug, T>(
    label: &str,
    cases: &[C],
    slots: Vec<Option<Result<T, String>>>,
) -> Vec<T> {
    if let Some((idx, msg)) = slots.iter().enumerate().find_map(|(i, s)| match s {
        Some(Err(m)) => Some((i, m)),
        _ => None,
    }) {
        panic!("{label} case #{idx} ({:?}) failed: {msg}", cases[idx]);
    }
    slots
        .into_iter()
        .enumerate()
        .map(|(idx, slot)| match slot {
            Some(Ok(v)) => v,
            Some(Err(msg)) => panic!("{label} case #{idx} ({:?}) failed: {msg}", cases[idx]),
            None => panic!(
                "{label} case #{idx} ({:?}) skipped after earlier failure",
                cases[idx]
            ),
        })
        .collect()
}
