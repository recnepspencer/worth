//! The one door through which a run of a partitioned computation reuses the
//! last run of the producer that runs it.
//!
//! A partition is skipped only if every fact its owner calls read last time,
//! observed now at this attempt's own snapshot, has the same content. The
//! comparator observes the native owner's dirty ordinals when the exact prior
//! and selected source qualify, otherwise every retained fact. It decides which
//! partitions are marked, charges carried calls and enters their facts as admitted
//! reads, and the next tree is built here from one disposition per
//! partition: carried, or gathered and computed again. The retained state
//! has no accessor outside this module, so every later way to skip work
//! enters through it.
//!
//! A run's result, its charged work and the partition a work ceiling names
//! never depend on reuse; request memory and checkpoint count do, since a
//! carried partition reserves its actual held objects rather than a full
//! run's gathers and scratch. Memory is physical, so its refusal may differ
//! with reuse or posture; work-ceiling stops and logical outcomes may not. Carried charges replay in a full
//! run's order among the calls made again, membership, item keys, gathers and
//! kernels by identity over the run's own items and partitions, then the
//! combines of the run's own tree, so a ceiling names the partition a full
//! run would name.

mod carriage;
mod native_reads;
pub(in crate::domain_computation::primary_graph) use native_reads::NativeComputationReads;
mod deposit;
mod next_tree;
mod observed;
pub use observed::full_partitioned_computation_preparations_on_this_thread_for_test;
pub(super) use observed::observe_full_preparation;
mod prepare;
mod prior_absence;
mod published_state;
pub(in crate::domain_computation) use published_state::observe_discarded;
pub(in crate::domain_computation::primary_graph) use published_state::observe_published;
pub use published_state::{
    discarded_computation_retention_on_this_thread_for_test,
    published_partitioned_computations_on_this_thread_for_test,
    WorthQueryPublishedComputationStateForTest,
};
mod recording;
mod retained;
mod sealing;
mod typed_prior;
pub(in crate::domain_computation::primary_graph) use deposit::ComputationDeposit;
pub(in crate::domain_computation::primary_graph) use prior_absence::CompletedComputationRetention;
pub(in crate::domain_computation) use prior_absence::{
    PriorAbsence, SealedComputationRetention, Suppression,
};
mod tree_attempt;
mod tree_edits;
mod tree_report;
mod tree_update;
pub(in super::super) use observed::observe_retained_full_tree;
pub use observed::partitioned_computation_tree_work_on_this_thread_for_test;
#[cfg(test)]
pub(in super::super) use observed::{observe_full_tree, FullTreeMapWork};
pub use tree_report::{
    WorthQueryPartitionedTreeMetrics, WorthQueryPartitionedTreeRebuildCause,
    WorthQueryPartitionedTreeRun,
};

use std::collections::BTreeSet;
use std::sync::Arc;

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;

pub(super) use next_tree::{ComputedIncremental, PreparedIncremental};
pub use observed::partitioned_computation_runs_on_this_thread_for_test;
pub(super) use recording::{observe_unretained, FullRecording};
pub(in crate::domain_computation::primary_graph) use retained::ComputationInstallation;
pub use retained::WorthQueryPartitionedComputationFullCause;
pub use retained::WorthQueryPartitionedComputationRunForTest;
pub(in crate::domain_computation) use retained::{
    ComputationPrior, RetainedComputation, SealedComputationRun,
};

use self::retained::{RetainedBasis, TypedPrior};
use super::remaining_work::RemainingWork;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationRead, FactMovement, Movement, WorthQueryApplicationFactKey,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader;

type Reader<'reader, 'runtime, Schema, Operation> =
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>;

/// How a retained fact stands at this attempt's snapshot. Only the fact
/// module's comparison says `Unchanged`; a fact that cannot be observed marks
/// its partition exactly as a changed one does.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeyMovement {
    Unchanged,
    Changed,
    NotObservable,
}

/// The comparator's own key to the reader's retained calls: no value of it
/// exists outside this module, so nothing else carries, observes or enters
/// a retained call.
pub(in crate::domain_computation::primary_graph) struct Comparator(());

const COMPARATOR: Comparator = Comparator(());

/// What the comparator decided before any owner call ran.
pub(super) enum Begun<Key, Item, Reduced> {
    /// Every partition runs. `basis` is what the run retains under, when a
    /// producer runs it.
    Full {
        basis: Result<RetainedBasisToken, Suppression>,
        cause: WorthQueryPartitionedComputationFullCause,
    },
    /// Only the marked partitions are gathered and computed.
    Incremental(IncrementalRun<Key, Item, Reduced>),
}

/// The basis a full run retains its state under, opaque outside this module.
pub(super) struct RetainedBasisToken(RetainedBasis);

/// A comparison of a retained run under the same basis: which of its calls
/// read a fact that moved, and the facts each call read.
pub(super) struct IncrementalRun<Key, Item, Reduced> {
    basis: RetainedBasis,
    prior: TypedPrior<Key, Item, Reduced>,
    /// Whether a fact the membership read moved.
    membership_moved: bool,
    /// The items a fact their key read moved for.
    moved_items: BTreeSet<PartitionItemId>,
    /// The partitions a fact their gathering read moved for.
    marked: BTreeSet<PartitionIdentity>,
    calls: Arc<crate::domain_computation::primary_graph::application_attempt::ComputationFactIndex>,
    validated_at: Option<worth_relational::facade::snapshots::SnapshotHandle>,
}

/// Compares the state the producer handed the reader with this attempt's
/// facts. `installation` names the installed owner, `input_digest` is the
/// input value's digest, already charged, and `remaining_work` is what the
/// digest left of the declared work.
///
/// Comparing a retained fact is charged to no one, because a full run would
/// not compare it, and charging it would make whether a later call passes
/// depend on reuse. The declared work bounds it instead: a state whose facts'
/// summed worst-case observation passes what that work still admits, or has
/// a fact nothing bounds, is not compared, and runs in full.
pub(super) fn begin<Key, Item, Reduced, Schema, Operation>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    reader: &mut Reader<'_, '_, Schema, Operation>,
    installation: &ComputationInstallation,
    input_digest: [u8; 32],
    remaining_work: RemainingWork,
) -> Result<
    Begun<Key, Item, Reduced>,
    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
>
where
    Key: Send + Sync + 'static,
    Item: Send + Sync + 'static,
    Reduced: Send + Sync + 'static,
{
    let prior = match reader.take_computation_prior() {
        Ok(prior) => prior,
        Err(reason) => {
            return Ok(Begun::Full {
                basis: Err(reason.suppression()),
                cause: reason.full_cause(),
            });
        }
    };
    let basis = RetainedBasis::new(
        installation.clone(),
        prior.edition,
        input_digest,
        reader.computation_value_bindings(&COMPARATOR),
    );
    let full = |basis, cause| {
        Ok(Begun::Full {
            basis: Ok(RetainedBasisToken(basis)),
            cause,
        })
    };
    if super::certification_reuse::disabled() {
        return full(basis, WorthQueryPartitionedComputationFullCause::Unretained);
    }
    let changes = reader.computation_changes(&COMPARATOR, &prior);
    let retained = match prior.retained {
        Ok(retained) => retained,
        Err(cause) => return full(basis, cause),
    };
    if let Some(cause) = basis.drift(&retained.basis) {
        return full(basis, cause);
    }
    if retained
        .facts
        .observation_work_bound()
        .is_none_or(|work| work > remaining_work.remaining())
    {
        return full(
            basis,
            WorthQueryPartitionedComputationFullCause::ObservationOverBudget,
        );
    }
    // The installation fixes the state's type, so its own owner's state
    // always downcasts.
    let prior = TypedPrior::from_state(Arc::clone(&retained));
    let mut run = IncrementalRun {
        basis,
        prior,
        membership_moved: false,
        moved_items: BTreeSet::new(),
        marked: BTreeSet::new(),
        calls: retained.facts.call_index(),
        validated_at: changes
            .as_ref()
            .map(|_| reader.computation_snapshot(&COMPARATOR)),
    };
    let all = changes
        .is_none()
        .then(|| retained.facts.facts())
        .into_iter()
        .flatten();
    let dirty = changes
        .iter()
        .flat_map(|ordinals| ordinals.iter())
        .filter_map(|ordinal| retained.facts.at_ordinal(*ordinal));
    for (key, fact, readers) in all.chain(dirty) {
        super::super::super::fact_verification_observation::retained_computation();
        #[cfg(test)]
        let _read = observed::RetainedFactComparison::enter();
        let movement = match reader.observe_retained(execution, &COMPARATOR, key) {
            Ok(observed) => match FactMovement::between(fact, &observed).movement() {
                Movement::Unmoved => KeyMovement::Unchanged,
                Movement::Moved => KeyMovement::Changed,
            },
            Err(denial) if matches!(denial.kind(),
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ExecutionRequest(_)
                | crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::Cancelled
                | crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::DeadlineExceeded
            ) => return Err(denial),
            Err(_) => KeyMovement::NotObservable,
        };
        let moved = movement != KeyMovement::Unchanged;
        for read in readers.reads() {
            if moved {
                match read {
                    ComputationRead::Membership => run.membership_moved = true,
                    ComputationRead::ItemKey(item) => {
                        run.moved_items.insert(item);
                    }
                    ComputationRead::Partition(partition) => {
                        run.marked.insert(partition);
                    }
                }
            }
        }
    }
    Ok(Begun::Incremental(run))
}

#[cfg(test)]
mod tests;

/// Enters the facts a carried call read as that call's admitted reads.
fn enter<'key, Schema, Operation>(
    reader: &mut Reader<'_, '_, Schema, Operation>,
    keys: impl IntoIterator<Item = &'key WorthQueryApplicationFactKey>,
    read: ComputationRead,
) {
    if reader.enter_native_carried(&COMPARATOR, read) {
        return;
    }
    for key in keys {
        reader.enter_carried(&COMPARATOR, key.clone(), read);
    }
}

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) fn sealed_run_for_lineage_test(
) -> SealedComputationRun {
    tests::sealed_run_for_lineage_test()
}
