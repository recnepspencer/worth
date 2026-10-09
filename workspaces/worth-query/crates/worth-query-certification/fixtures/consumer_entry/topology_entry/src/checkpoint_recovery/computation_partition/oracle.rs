//! The oracle of partition reuse: a program whose producer keeps a region
//! output over a set of entries, and how its tests seed, edit and demand it.
//! Every run the producer's decision makes is left in a room with how it ran
//! and which of the owner's calls it entered.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use worth_query_host::facade::application_contribution::{
    published_partitioned_computations_on_this_thread_for_test as published_states,
    WorthQueryComputationPartitionMembers, WorthQueryComputationPartitionPlan,
    WorthQueryComputationPartitionView, WorthQueryDeterministicReducer,
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
    WorthQueryPartitionedComputationRun, WorthQueryPublishedComputationStateForTest,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandProgress,
    WorthQueryApplicationRequest,
};
use worth_query_host::facade::primary_graph::partitioned_computation_runs_on_this_thread_for_test as runs_on_this_thread;

use super::demand::RegionTotalsDemandBinding;
use super::entry_edit::{EntryEdit, EntryEditBinding, EntryEditHandler};
use super::facts::{self, Entry, EntryData, Graph, InputDenial, Reader, RegionEntry, Set};
use super::output_producer::{
    RegionOutputDemand, RegionOutputProducer, RegionOutputProvider, RegionOutputReadiness,
};
use super::region_output::{
    RegionOutputBinding, RegionOutputHandler, TotalRegionOutput, DECISION_FACT_BUDGET, LARGEST_SET,
};
use super::*;

mod absence;
mod advancement_custody;
mod counts;
mod cutoff;
mod handler_absence;
mod installation;
mod republication;
mod restoration;
mod seeded_absence;
mod tree_count;
mod tree_stop;
mod tree_work;
use installation::{
    install, install_with_reuse, Application, OracleProgram, Request, EVEN_Y, ODD_Y, SCOPE,
};
mod branch_sharing;
mod differential;
mod parallel_history_reuse;
mod program;
mod worker_axis;

use program::{OracleRoot, RegionArtifact, RegionConnection, TOTALS_RETAINED_BYTES, TOTALS_WORK};

/// What one entry may cost the computation: its digest, its key's encoding
/// and routing, its kernel and its share of the combines.
const WORK_PER_ENTRY: usize = 512;
/// The input's digest.
const WORK_BESIDE_ENTRIES: usize = 4_096;
/// The width every other operation of the program fits, the host's default.
const WIDTH_BESIDE_DECISION: usize = 4_096;

/// The totals of one region output, declared for the largest set its
/// decision reads.
struct OracleTotals<const WORK: usize = TOTALS_WORK>;
impl<const WORK: usize> ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>
    for OracleTotals<WORK>
{
    type Input = RegionEntries;
    type Output = RegionArtifact;
    type Partition = RegionKey;
    type Reuse = NoWarmStart;
    type Stopped = RegionStopped;
    const IDENTITY: &'static str = "checkpoint-region-output-totals";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(WORK, TOTALS_RETAINED_BYTES);
}

thread_local! { static KERNEL_CHARGES: std::cell::RefCell<Vec<u64>> = const { std::cell::RefCell::new(Vec::new()) }; }

fn kernel_charges() -> Vec<u64> {
    KERNEL_CHARGES.with(|charges| std::mem::take(&mut *charges.borrow_mut()))
}
static COMBINES: AtomicUsize = AtomicUsize::new(0);

static PLANS: AtomicUsize = AtomicUsize::new(0);
static KEYS: AtomicUsize = AtomicUsize::new(0);
static GATHERS: AtomicUsize = AtomicUsize::new(0);
static KERNELS: AtomicUsize = AtomicUsize::new(0);

/// How often a decision's run entered each of the owner's calls.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OwnerCalls {
    plans: usize,
    keys: usize,
    gathers: usize,
    kernels: usize,
}

fn take_calls() -> OwnerCalls {
    OwnerCalls {
        plans: PLANS.swap(0, Ordering::Relaxed),
        keys: KEYS.swap(0, Ordering::Relaxed),
        gathers: GATHERS.swap(0, Ordering::Relaxed),
        kernels: KERNELS.swap(0, Ordering::Relaxed),
    }
}

/// Totals each region in entry order, then the regions. An even region also
/// sums the weight its set lends it, so the weight is a fact every even
/// region gathers.
struct OracleOwner<const MODE: u8 = 0>;
impl<const WORK: usize, const MODE: u8>
    WorthQueryPartitionedComputationOwner<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        OracleTotals<WORK>,
    > for OracleOwner<MODE>
{
    type Operation = TotalRegionOutput;
    type Item = Entry;
    type Gathered = Vec<EntryData>;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        set: &Set,
    ) -> Result<WorthQueryComputationPartitionPlan<Entry>, InputDenial> {
        PLANS.fetch_add(1, Ordering::Relaxed);
        Ok(facts::entries(reader, set)?)
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        _: &Set,
        entry: &Entry,
    ) -> Result<RegionKey, InputDenial> {
        KEYS.fetch_add(1, Ordering::Relaxed);
        let region = facts::region(reader, entry)?;
        let memberships = if MODE == 1 {
            u32::try_from(facts::membership_count(reader, entry)?).unwrap()
        } else {
            0
        };
        Ok(RegionKey(region + memberships))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        set: &Set,
        partition: WorthQueryComputationPartitionMembers<'_, RegionKey, Entry>,
    ) -> Result<Vec<EntryData>, InputDenial> {
        GATHERS.fetch_add(1, Ordering::Relaxed);
        let mut gathered = Vec::new();
        if partition.key().0.is_multiple_of(2) {
            gathered.push(EntryData {
                value: facts::weight(reader, set)?,
                work: 1,
                fault: None,
            });
        }
        gathered.extend(facts::gathered(reader, partition.items())?);
        Ok(gathered)
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, RegionKey, Vec<EntryData>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        KERNELS.fetch_add(1, Ordering::Relaxed);
        let outcome = owner::total_region(partition.key().0, partition.gathered(), checkpoint);
        if outcome.is_ok() {
            // A completed region passed every declared entry checkpoint.
            let work = partition
                .gathered()
                .iter()
                .map(|entry| entry.work as u64)
                .sum();
            KERNEL_CHARGES.with(|charges| charges.borrow_mut().push(work));
        }
        outcome
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(
            || -0.0,
            if MODE == 4 {
                tree_count::faulting_sum
            } else {
                tree_count::sum
            },
        )
    }

    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}

/// One run of the producer's decision: the total's bits and what the run was
/// charged, or its denial; how it ran; and the owner's calls it entered.
#[derive(Debug)]
struct OracleRun {
    published: Vec<WorthQueryPublishedComputationStateForTest>,
    outcome: Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>,
    runs: Vec<WorthQueryPartitionedComputationRun>,
    calls: OwnerCalls,
    tree_runs:
        Vec<worth_query_host::facade::application_contribution::WorthQueryPartitionedTreeRun>,
    combines: usize,
    tree_nodes: usize,
    placement: worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest,
}

/// The runs of the current demand. The tests that fill it hold the
/// checkpoint recovery guard.
static ROOM: Mutex<Vec<OracleRun>> = Mutex::new(Vec::new());

fn room() -> MutexGuard<'static, Vec<OracleRun>> {
    ROOM.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The one room take reconciles every captured run before exposing it.
fn take_runs(
    serial_tree: Option<
        &[worth_query_host::facade::application_contribution::WorthQueryPartitionedTreeRun],
    >,
) -> Vec<OracleRun> {
    let runs = std::mem::take(&mut *room());
    for run in &runs {
        tree_work::assert_reconciled(run, tree_work::stop_kind(run, serial_tree));
    }
    runs
}

/// Seeds the entry numbered `number` as a member of each of `sets`.
fn seed_entry(graph: &mut Graph, sets: &[&str], number: usize, entry: RegionEntry) {
    let key = format!("oracle-entry-{number}");
    facts::seed_entry(graph, &key, &entry);
    for set in sets {
        facts::seed_member(graph, set, &key);
    }
}

/// Demands the scope's output until it settles: the producer's contacts in
/// the demand, and the runs its decisions made.
fn demand<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
) -> (usize, Vec<OracleRun>) {
    demand_reconciled(request, application, None)
}

fn demand_reconciled<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
    serial_tree: Option<
        &[worth_query_host::facade::application_contribution::WorthQueryPartitionedTreeRun],
    >,
) -> (usize, Vec<OracleRun>) {
    room().clear();
    published_states();
    let mut demand = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<REUSE, WORK, RUNS, MODE>, RegionConnection>(
            application,
        )
        .expect("the region output demand starts");
    let settled = (0..256)
        .find_map(|_| {
            match demand.advance(request).unwrap_or_else(|denial| {
                panic!(
                    "the region output demand advances: {denial:?}; runs: {:?}",
                    *room()
                )
            }) {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
            }
        })
        .expect("the region output demand settles");
    let contacts = settled.producer_contacts_in_this_demand();
    let mut runs = take_runs(serial_tree);
    if let Some(last) = runs.last_mut() {
        last.published = published_states();
    }
    (contacts, runs)
}

/// Commits one entry edit.
fn edit<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
    edit: EntryEdit,
    command: u64,
) {
    let made = format!("{edit:?}");
    let outcome = request
        .mutate(edit.commanded(command))
        .without_source()
        .idempotency(&command)
        .execute_in_program::<OracleProgram<REUSE, WORK, RUNS, MODE>>(application);
    assert!(
        matches!(
            &outcome,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "the edit {made} commits: {outcome:?}"
    );
}

/// Moves the scope's ordinate to `y`, which names the set its output totals.
fn adjust<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>(
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
    y: u64,
    command: u64,
) {
    let observed = request
        .query(PlanarRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .expect("the scope's source is readable");
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: SCOPE.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&command)
        .execute_performed::<OracleProgram<REUSE, WORK, RUNS, MODE>, OracleRoot>(application)
        .expect("the scope's ordinate moves");
}

fn at_demand_scope(mut edit: EntryEdit) -> EntryEdit {
    edit.scope_key = SCOPE.to_owned();
    edit
}

fn assert_published_state(kept: &[OracleRun], fresh: &[OracleRun]) {
    let kept = kept.last().unwrap();
    let fresh = fresh.last().unwrap();
    assert_eq!(
        kept.published.len(),
        fresh.published.len(),
        "published state count"
    );
    for (kept, fresh) in kept.published.iter().zip(&fresh.published) {
        assert!(
            kept.same_fields::<RegionKey, Entry, f64>(
                fresh,
                |a, b| a.0 == b.0,
                Entry::same_binding_as
            ),
            "published retained fields differ: {kept:?}, {fresh:?}"
        );
    }
}
