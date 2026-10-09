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
mod counts;
mod courtroom;
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
mod application;
use application::*;

/// One run of the producer's decision: the total's bits and what the run was
/// charged, or its denial; how it ran; and the owner's calls it entered.
#[derive(Debug)]
struct OracleRun {
    published: Vec<WorthQueryPublishedComputationStateForTest>,
    outcome: Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>,
    runs: Vec<WorthQueryPartitionedComputationRun>,
    full_preparations: Vec<worth_query_host::facade::application_contribution::WorthQueryPartitionedComputationFullCause>,
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

mod demand;
use demand::*;

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
