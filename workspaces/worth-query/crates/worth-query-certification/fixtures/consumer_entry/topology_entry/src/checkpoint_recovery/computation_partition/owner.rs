//! The partitioned owner of the region totals, and the demand that runs it
//! under a real request.

use std::sync::{Mutex, MutexGuard, PoisonError};

#[cfg(feature = "test-query-execution-observer")]
use worth_foundational::facade::ExecutionReport;
#[cfg(feature = "test-query-execution-observer")]
use worth_query_host::facade::application_contribution::WorthQueryPartitionedComputationRun;
use worth_query_host::facade::application_contribution::{
    WorthQueryComputationPartitionMembers, WorthQueryComputationPartitionPlan,
    WorthQueryComputationPartitionView, WorthQueryDeterministicReducer,
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
#[cfg(feature = "test-query-execution-observer")]
use worth_query_host::facade::primary_graph::partitioned_computation_runs_on_this_thread_for_test as runs_on_this_thread;

#[cfg(feature = "test-query-execution-observer")]
pub(super) mod calls;

use super::demand::{RegionTotalsDemand, RegionTotalsHandler, TotalRegions};
use super::facts::{self, Entry, EntryData, InputDenial, Reader, RegionFault, Set, Sets};
use super::*;

/// Sums each region's values in entry identity order, then the regions' sums.
/// Addition's identity is negative zero: adding it changes no sum, and keeps
/// the sign of a sum that is itself a zero.
pub(super) struct RegionTotalsOwner;
impl<Computation>
    WorthQueryPartitionedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, Computation>
    for RegionTotalsOwner
where
    Computation: ApplicationManagedComputation<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Input = RegionEntries,
        Partition = RegionKey,
    >,
{
    type Operation = TotalRegions;
    type Item = Entry;
    type Gathered = Vec<EntryData>;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        set: &Set,
    ) -> Result<WorthQueryComputationPartitionPlan<Entry>, InputDenial> {
        #[cfg(feature = "test-query-execution-observer")]
        calls::record(0);
        Ok(facts::entries(reader, set)?)
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        _: &Set,
        entry: &Entry,
    ) -> Result<RegionKey, InputDenial> {
        #[cfg(feature = "test-query-execution-observer")]
        calls::record(1);
        Ok(RegionKey(facts::region(reader, entry)?))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        _: &Set,
        partition: WorthQueryComputationPartitionMembers<'_, RegionKey, Entry>,
    ) -> Result<Vec<EntryData>, InputDenial> {
        #[cfg(feature = "test-query-execution-observer")]
        calls::record(2);
        Ok(facts::gathered(reader, partition.items())?)
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, RegionKey, Vec<EntryData>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        #[cfg(feature = "test-query-execution-observer")]
        calls::record(3);
        total_region(partition.key().0, partition.gathered(), checkpoint)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(|| -0.0, |left, right| left + right)
    }

    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}

/// One region's kernel: sums its entries' values in the order it holds them,
/// charging each entry's work before reading it.
pub(super) fn total_region(
    region: u32,
    entries: &[EntryData],
    checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
    #[cfg(feature = "test-query-execution-observer")]
    super::worker_axis::enter_overlapped();
    let mut total = -0.0;
    for entry in entries {
        checkpoint.advance(entry.work)?;
        match entry.fault {
            Some(RegionFault::Refuse) => {
                return Err(WorthQueryManagedComputationDenial::Owner(region))
            }
            Some(RegionFault::Panic) => panic!("the region kernel panics"),
            Some(RegionFault::Probe(_)) | None => total += entry.value,
        }
    }
    Ok(total)
}

/// The partitioned computation bound to its partitioned owner.
pub(super) struct PartitionedOwner;
impl RegionTotalsBinding for PartitionedOwner {
    type Computation = RegionTotals;

    fn install(setup: &mut Setup<'_>) -> Installed {
        running::<RegionTotals, _>(setup, RegionTotalsOwner)
    }
}

/// The same owner under a computation whose declared bytes are the largest
/// a declaration can name.
pub(super) struct UnboundedBytesOwner;
impl RegionTotalsBinding for UnboundedBytesOwner {
    type Computation = UnboundedBytesTotals;

    fn install(setup: &mut Setup<'_>) -> Installed {
        running::<UnboundedBytesTotals, _>(setup, RegionTotalsOwner)
    }
}

pub(super) type Setup<'setup> = WorthQueryApplicationContributionSetup<'setup, CheckpointSchema>;
pub(super) type Installed = Result<
    RegionTotalsHandler<CheckpointSchema>,
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
>;

/// A completed run: the total's bits, what the run was charged, and, where
/// the observer is built, execution's report of the run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegionTotal {
    pub(super) bits: u64,
    pub(super) charged_work: u64,
    #[cfg(feature = "test-query-execution-observer")]
    pub(super) report: ExecutionReport,
}

pub(super) type RegionOutcome = Result<RegionTotal, WorthQueryPartitionedComputationDenial<u32>>;

/// What each decision's run came to. No owner reads it: an owner's input is
/// the facts of the demanded set. The tests that use it hold the checkpoint
/// recovery guard.
struct RegionRoom {
    outcomes: Vec<RegionOutcome>,
    /// The partitions whose kernel the last demand entered.
    kernels: usize,
}

static ROOM: Mutex<RegionRoom> = Mutex::new(RegionRoom {
    outcomes: Vec::new(),
    kernels: 0,
});

fn room() -> MutexGuard<'static, RegionRoom> {
    ROOM.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(feature = "test-query-execution-observer")]
pub(super) fn take_outcomes() -> Vec<RegionOutcome> {
    std::mem::take(&mut room().outcomes)
}

/// A kernel says it was entered.
pub(super) fn enter_kernel() {
    room().kernels += 1;
}

/// The partitions whose kernel the last demand entered.
pub(super) fn kernels_entered() -> usize {
    room().kernels
}

/// Installs the computation's partitioned owner and returns the demand's
/// handler, which runs the computation over the demanded set at every
/// decision, reading through its own reader with its request's execution.
pub(super) fn running<Computation, Owner>(setup: &mut Setup<'_>, owner: Owner) -> Installed
where
    Computation: ApplicationManagedComputation<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Input = RegionEntries,
    >,
    Owner: WorthQueryPartitionedComputationOwner<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Computation,
        Operation = TotalRegions,
        Output = f64,
        Stopped = u32,
    >,
{
    let installed =
        setup.partitioned_computation::<PlanarFinalOutputFeature, Computation, _>(owner)?;
    Ok(RegionTotalsHandler::running(move |reader, set| {
        let outcome: RegionOutcome = (|| {
            let computed = installed
                .prepare(reader, set)?
                .compute(reader.managed_computation_execution())?;
            let charged_work = computed.charged_work();
            #[cfg(feature = "test-query-execution-observer")]
            let report = observed_report();
            Ok(RegionTotal {
                bits: computed.complete()?.to_bits(),
                charged_work,
                #[cfg(feature = "test-query-execution-observer")]
                report,
            })
        })();
        room().outcomes.push(outcome);
    }))
}

/// Execution's report of the decision's one run, which ran in full. How a
/// run ran is the observer's, never the handler's value.
#[cfg(feature = "test-query-execution-observer")]
fn observed_report() -> ExecutionReport {
    let runs = runs_on_this_thread();
    assert_eq!(runs.len(), 1, "one run per decision");
    assert_eq!(runs[0].0, WorthQueryPartitionedComputationRun::Full(
        worth_query_host::facade::application_contribution::WorthQueryPartitionedComputationFullCause::Unretained));
    runs[0]
        .1
        .expect("the unretained full run reports its execution work")
}

/// The region totals under their partitioned owner.
pub(super) fn with_region_totals(
    sets: Sets<'_>,
    test: impl FnOnce(&mut dyn FnMut(&str) -> RegionOutcome),
) {
    with_totals::<PartitionedOwner>(sets, test);
}

/// Installs the binding's program over the seeded `sets` and hands the test a
/// demand: each call totals the named set inside one committed mutation and
/// returns what that decision's run came to.
pub(super) fn with_totals<Binding: RegionTotalsBinding>(
    sets: Sets<'_>,
    test: impl FnOnce(&mut dyn FnMut(&str) -> RegionOutcome),
) {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program_with_seed::<RegionTotalsProgram<Binding>>(
        None,
        Default::default(),
        32,
        128 * 1_024 * 1_024,
        1_000_000,
        |graph| {
            support::seed_cycle(graph);
            facts::seed(graph, sets);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut demands = 0_u64;
    test(&mut |set| {
        demands += 1;
        {
            let mut room = room();
            room.outcomes.clear();
            room.kernels = 0;
        }
        let observed = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the adjusted source is readable");
        // Every demand moves the source, so every decision is a new one.
        let outcome = request
            .mutate(RegionTotalsDemand {
                scope_key: "anchor-a".to_owned(),
                entries: set.to_owned(),
                replacement_y: length(2 + demands % 2),
            })
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(&demands)
            .execute_in_program::<RegionTotalsProgram<Binding>>(&application);
        assert!(
            matches!(
                &outcome,
                Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
            ),
            "the demand commits whatever the totals came to: {outcome:?}"
        );
        let mut outcomes = std::mem::take(&mut room().outcomes);
        assert_eq!(outcomes.len(), 1, "one decision runs the totals once");
        outcomes.remove(0)
    });
}
