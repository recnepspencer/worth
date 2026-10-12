//! The worker axis: the same demands run serially, on one worker, on two, on
//! the machine's width and on twice it, and on leases whose every full
//! reduction is certified against a seeded perturbed backend. Every placement
//! keeps the serial run's bits, its charged work, its work boundary and its
//! least failing partition. Memory and deadline boundaries move with the
//! worker count; no case here meets one, and none is compared. Every demand
//! on two workers or more, completed or denied, holds two kernels at once.

use std::num::NonZeroUsize;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use worth_foundational::facade::ExecutionFallbackCause;
use worth_query_host::facade::application_contribution::WorthQueryPartitionedComputationDenial;
use worth_query_host::facade::primary_graph::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};

use super::execution::{association_sensitive_entries, entry, heavy_regions, several_refusals};
use super::facts::{self, RegionEntry};
use super::owner::{with_region_totals, RegionOutcome};

worth_execution::needs_os_threads! {
use worth_query_host::facade::primary_graph::test_execution_workers_for_test;

/// The perturbation seeds of the certified placements.
const SEEDS: [u64; 3] = [0x6_7000_0001, 0x6_7000_0002, 0x6_7000_0003];

/// Every placement of the axis, serial first. The authority admits twice the
/// machine's width, so the widest lease asks for more workers than the
/// machine has.
pub(super) fn axis() -> Vec<Placement> {
    let width = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
    assert!(
        test_execution_workers_for_test().get() >= 2 * width,
        "the authority admits twice the machine's width"
    );
    let workers = |count: usize| NonZeroUsize::new(count).expect("a worker count is not zero");
    let mut axis = vec![Placement::Serial];
    axis.extend([1, 2, width, 2 * width].map(|count| Placement::Leased(workers(count))));
    axis.extend(
        [2, width, 2 * width]
            .into_iter()
            .zip(SEEDS)
            .map(|(count, seed)| Placement::Certified {
                workers: workers(count),
                seed,
            }),
    );
    axis
}
}

/// The workers `placement` asks for: none when it holds no lease.
pub(super) fn workers_of(placement: Placement) -> usize {
    match placement {
        Placement::Leased(workers) | Placement::Certified { workers, .. } => workers.get(),
        Placement::Serial | Placement::World => 0,
    }
}

/// The kernels of the demand at hand. While a demand is armed, its first
/// kernel waits for a second to enter, so a run that placed a second worker
/// holds both at once, and the two have met. A run on one worker never meets:
/// its first kernel gives up after [`PATIENCE`], and only then does a second
/// enter. Counting entries alone cannot tell the two apart.
struct Overlap {
    armed: bool,
    entered: usize,
    met: bool,
}

const DISARMED: Overlap = Overlap {
    armed: false,
    entered: 0,
    met: false,
};

static OVERLAP: (Mutex<Overlap>, Condvar) = (Mutex::new(DISARMED), Condvar::new());

/// How long a lone kernel waits for a second. A run that placed a second
/// worker never waits it out.
// Hang guard for the kernel-overlap rendezvous; elapsed time is not the fact under test.
const PATIENCE: Duration = Duration::from_secs(30);

fn overlap() -> MutexGuard<'static, Overlap> {
    OVERLAP.0.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Arms the overlap until dropped, then requires that two kernels met
/// while it was armed, whatever the demands came to. Only a demand under the
/// checkpoint recovery guard arms it.
pub(super) struct Overlapping;
impl Overlapping {
    pub(super) fn arm() -> Self {
        *overlap() = Overlap {
            armed: true,
            ..DISARMED
        };
        Self
    }
}
impl Drop for Overlapping {
    fn drop(&mut self) {
        let met = std::mem::replace(&mut *overlap(), DISARMED).met;
        if !std::thread::panicking() {
            assert!(met, "two-worker kernel-overlap rendezvous did not occur");
        }
    }
}

/// A region kernel enters. While its demand is armed, the first kernel waits
/// for a second, and they meet if the second enters before it gives up.
pub(super) fn enter_overlapped() {
    let mut overlap = overlap();
    if !overlap.armed {
        return;
    }
    overlap.entered += 1;
    let first = overlap.entered == 1;
    OVERLAP.1.notify_all();
    let (mut overlap, waited) = OVERLAP
        .1
        .wait_timeout_while(overlap, PATIENCE, |overlap| overlap.entered < 2)
        .unwrap_or_else(PoisonError::into_inner);
    overlap.met |= first && !waited.timed_out();
}

/// Runs `test` with this thread's managed computations at `placement`, and
/// returns them to the World's placement after, even when `test` panics.
pub(super) fn placed<T>(placement: Placement, test: impl FnOnce() -> T) -> T {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    struct Restore(Placement);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
        }
    }
    let _restore = Restore(place(placement));
    test()
}

/// What the axis compares of a demand: the total's bits and charged work, or
/// its denial.
type Compared = Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>;

/// Demands each of `demands` from `sets` under every placement, and requires
/// each placement's outcomes to be the serial ones. A completed total says
/// whether its run held a lease, so a placement that never reached the run
/// fails here. On two workers or more each demand, denied ones too, must
/// hold two kernels at once, so one that ran on one worker fails.
/// Returns the serial outcomes.
fn across_the_axis(
    sets: facts::Sets<'_>,
    demands: &[&str],
    placements: Vec<Placement>,
) -> Vec<Compared> {
    let mut serial: Option<Vec<Compared>> = None;
    for placement in placements {
        let parallel = workers_of(placement) >= 2;
        let outcomes: Vec<RegionOutcome> = placed(placement, || {
            let mut outcomes = Vec::new();
            with_region_totals(sets, |demand| {
                outcomes.extend(demands.iter().map(|name| {
                    let _overlapping = parallel.then(Overlapping::arm);
                    demand(name)
                }));
            });
            outcomes
        });
        for total in outcomes.iter().flatten() {
            let expected = if cfg!(target_family = "wasm") {
                Some(ExecutionFallbackCause::PlatformSerial)
            } else if placement == Placement::Serial {
                Some(ExecutionFallbackCause::PolicySerial)
            } else if workers_of(placement) == 1 {
                Some(ExecutionFallbackCause::WorkerLimit)
            } else {
                None
            };
            assert_eq!(
                total.report.fallback(),
                expected,
                "{placement:?} places the run"
            );
            if parallel {
                assert_eq!(
                    total.report.fallback(),
                    None,
                    "{placement:?} runs in parallel"
                );
                assert!(
                    total.report.physical().active_workers_high_watermark() > 1,
                    "{placement:?} holds two workers at once"
                );
            }
        }
        let compared = outcomes
            .into_iter()
            .map(|outcome| outcome.map(|total| (total.bits, total.charged_work)))
            .collect::<Vec<_>>();
        match &serial {
            None => serial = Some(compared),
            Some(serial) => assert_eq!(&compared, serial, "{placement:?} keeps the serial run"),
        }
    }
    serial.expect("the axis has a serial placement")
}

worth_execution::needs_os_threads! {
#[test]
fn the_floating_point_sum_keeps_its_bits_and_its_work_at_every_worker_count() { assert_floating_point(axis()); }
}

worth_execution::needs_os_threads! {
#[cfg(panic = "unwind")]
#[test]
fn the_least_failing_partition_is_the_same_at_every_worker_count() {
    let (entries, least) = several_refusals();
    assert_least_failure(axis(), entries, least);
}
}

worth_execution::needs_os_threads! {
#[test]
fn the_work_boundary_is_the_same_at_every_worker_count() { assert_work_boundary(axis()); }
}

#[test]
fn the_floating_point_sum_keeps_its_bits_and_its_work_at_one_worker() {
    assert_floating_point(one_worker());
}

fn assert_floating_point(placements: Vec<Placement>) {
    let sensitive = association_sensitive_entries();
    let exact = (1..=40_u32)
        .map(|id| entry(u64::from(id), id % 7, f64::from(id)))
        .collect::<Vec<_>>();
    let sets: facts::Sets<'_> = &[("sensitive", &sensitive), ("exact", &exact)];
    let serial = across_the_axis(sets, &["sensitive", "exact"], placements);
    assert!(f64::from_bits(serial[0].as_ref().unwrap().0).is_finite());
    assert_eq!(serial[1].as_ref().unwrap().0, 820.0_f64.to_bits());
}

#[test]
fn the_work_boundary_is_the_same_at_one_worker() {
    assert_work_boundary(one_worker());
}

fn assert_work_boundary(placements: Vec<Placement>) {
    let (heavy, exhausted) = heavy_regions();
    let mut reversed = heavy.clone();
    reversed.reverse();
    let light = heavy
        .iter()
        .map(|entry| RegionEntry { work: 1, ..*entry })
        .collect::<Vec<_>>();
    let sets: facts::Sets<'_> = &[
        ("heavy", &heavy),
        ("reversed", &reversed),
        ("light", &light),
    ];
    let [heavy, reversed, light] = <[Compared; 3]>::try_from(across_the_axis(
        sets,
        &["heavy", "reversed", "light"],
        placements,
    ))
    .unwrap();
    let exhausted = exhausted.expect_err("the work boundary is a denial");
    assert_eq!(heavy.err().as_ref(), Some(&exhausted));
    assert_eq!(reversed.err().as_ref(), Some(&exhausted));
    assert_eq!(light.unwrap().0, 3.0_f64.to_bits());
}

#[test]
fn the_least_failing_partition_is_the_same_at_one_worker() {
    let (mut entries, least) = several_refusals();
    // Typed refusals pin canonical failure order without needing unwinding.
    for entry in &mut entries {
        if matches!(entry.fault, Some(facts::RegionFault::Panic)) {
            entry.fault = Some(facts::RegionFault::Refuse);
        }
    }
    assert_least_failure(one_worker(), entries, least);
}

fn assert_least_failure(
    placements: Vec<Placement>,
    descending: Vec<RegionEntry>,
    least: RegionOutcome,
) {
    let mut ascending = descending.clone();
    ascending.reverse();
    let sets: facts::Sets<'_> = &[("descending", &descending), ("ascending", &ascending)];
    let least = least.expect_err("the least failure is a denial");
    for outcome in across_the_axis(sets, &["descending", "ascending"], placements) {
        assert_eq!(outcome.err().as_ref(), Some(&least));
    }
}

fn one_worker() -> Vec<Placement> {
    vec![Placement::Serial, Placement::Leased(NonZeroUsize::MIN)]
}
