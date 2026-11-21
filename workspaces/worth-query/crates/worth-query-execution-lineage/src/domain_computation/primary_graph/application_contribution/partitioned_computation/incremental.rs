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

pub mod carriage;
pub mod item_bounds;
pub mod native_reads;
pub use native_reads::NativeComputationReads;
pub mod deposit;
pub mod observed;
pub use observed::full_partitioned_computation_preparations_on_this_thread_for_test;
pub use observed::observe_full_preparation;
use super::preparation_work::PreparationWork;
pub mod prior_absence;
pub mod published_state;
pub use published_state::observe_discarded;
pub use published_state::observe_published;
pub use published_state::{
    discarded_computation_retention_on_this_thread_for_test,
    published_partitioned_computations_on_this_thread_for_test,
    WorthQueryPublishedComputationStateForTest,
};
pub mod carried_partitions;
pub mod partition_metadata;
pub mod recording;
pub mod retained;
pub mod sealing;
pub mod typed_prior;
pub use deposit::ComputationDeposit;
pub use prior_absence::CompletedComputationRetention;
pub use prior_absence::{
    PriorAbsence, SealedComputationRetention, Suppression,
};
pub mod tree_attempt;
pub mod tree_edits;
pub mod tree_membership;
pub mod tree_report;
pub mod tree_update;
pub use observed::observe_retained_full_tree;
pub use observed::partitioned_computation_tree_work_on_this_thread_for_test;
pub use tree_report::{
    WorthQueryPartitionedTreeAccounting, WorthQueryPartitionedTreeMetrics,
    WorthQueryPartitionedTreeRebuildCause, WorthQueryPartitionedTreeRun,
};

use std::collections::BTreeSet;
use std::sync::Arc;

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;

pub use observed::partitioned_computation_runs_on_this_thread_for_test;
pub use recording::{observe_unretained, FullRecording};
pub use retained::ComputationInstallation;
pub use retained::WorthQueryPartitionedComputationFullCause;
pub use retained::WorthQueryPartitionedComputationRunForTest;
pub use retained::{
    ComputationPrior, RetainedComputation, SealedComputationRun,
};

use self::retained::{RetainedBasis, TypedPrior};
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationRead, FactMovement, Movement, WorthQueryApplicationFactKey,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader;

pub type Reader<'reader, 'runtime, Schema, Operation> =
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
pub struct Comparator(());

pub const COMPARATOR: Comparator = Comparator(());

/// What the comparator decided before any owner call ran.
pub enum Begun<Key, Item, Reduced> {
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
pub struct RetainedBasisToken(RetainedBasis);

/// A comparison of a retained run under the same basis: which of its calls
/// read a fact that moved, and the facts each call read.
pub struct IncrementalRun<Key, Item, Reduced> {
    pub basis: RetainedBasis,
    pub prior: TypedPrior<Key, Item, Reduced>,
    /// Whether a fact the membership read moved.
    pub membership_moved: bool,
    /// The items a fact their key read moved for.
    pub moved_items: BTreeSet<PartitionItemId>,
    /// The partitions a fact their gathering read moved for.
    pub marked: BTreeSet<PartitionIdentity>,
    pub calls: Arc<crate::domain_computation::primary_graph::application_attempt::ComputationFactIndex>,
    pub validated_at: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    pub rebase_changes: Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::ComputationChanges,
    >,
}

/// Compares the state the producer handed the reader with this attempt's
/// facts. `installation` names the installed owner, `input_digest` is the
/// input value's digest, already admitted by the original request.
///
/// Comparing a retained fact is charged to no one, because a full run would
/// not compare it, and charging it would make whether a later call passes
/// depend on reuse. The original request's policy bounds it instead: a state
/// whose worst-case observation exceeds that policy, or whose facts have no
/// finite observation bound, runs in full.
pub fn begin<Key, Item, Reduced, Schema, Operation>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    reader: &mut Reader<'_, '_, Schema, Operation>,
    installation: &ComputationInstallation,
    input_digest: [u8; 32],
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
    let rebase_changes = changes.clone();
    let retained = match prior.retained {
        Ok(retained) => retained,
        Err(cause) => return full(basis, cause),
    };
    if let Some(cause) = basis.drift(&retained.basis) {
        return full(basis, cause);
    }
    if execution.has_accounting()
        && retained.facts.observation_work_bound().is_none_or(|work| {
            execution
                .work_ceiling()
                .is_some_and(|ceiling| work > ceiling)
        })
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
        rebase_changes,
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
        .flat_map(|changes| changes.ordinals().iter())
        .filter_map(|ordinal| retained.facts.at_ordinal(*ordinal));
    let structural = |key: &WorthQueryApplicationFactKey| {
        matches!(
            key,
            WorthQueryApplicationFactKey::StructuralWalk(_)
                | WorthQueryApplicationFactKey::Aggregate(_)
        )
    };
    let ordinary = all.chain(dirty).filter(|(key, _, _)| !structural(key));
    // Keep the existing sparse path when the sealed index has no structural keys.
    let issued = (retained.facts.has_aggregate() || retained.facts.has_structural_walk())
        .then(|| retained.facts.facts())
        .into_iter()
        .flatten()
        .filter(|(key, _, _)| structural(key));
    for (key, fact, readers) in ordinary.chain(issued) {
        super::super::super::fact_verification_observation::retained_computation();
        let _read = observed::RetainedFactComparison::enter();
        let movement = match reader.issue_retained(execution, &COMPARATOR, key, fact) {
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

/// Enters the facts a carried call read as that call's admitted reads.
pub fn enter<'key, Schema, Operation>(
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

mod court_doors_for_test;

pub use tree_membership::TreeTarget;
