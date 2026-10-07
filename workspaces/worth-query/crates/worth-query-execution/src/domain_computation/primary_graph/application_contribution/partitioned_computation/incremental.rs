//! The one door through which a run of a partitioned computation reuses the
//! last run of the producer that runs it.
//!
//! A partition is skipped only if every fact its owner calls read last time,
//! observed now at this attempt's own snapshot, has the same content. The
//! comparator here observes every retained fact, decides which partitions
//! are marked, charges the carried calls and enters their facts as admitted
//! reads, and the next tree is built here from one disposition per
//! partition: carried, or gathered and computed again. The retained state
//! has no accessor outside this module, so every later way to skip work
//! enters through it.
//!
//! An outcome never depends on reuse. Carried charges replay in a full run's
//! order among the calls made again, membership, item keys, gathers and
//! kernels by identity over the run's own items and partitions, then the
//! combines of the run's own tree, so a ceiling names the partition a full
//! run would name.

mod next_tree;
mod observed;
mod prepare;
mod recording;
mod retained;
mod tree_update;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;

pub(super) use next_tree::{ComputedIncremental, PreparedIncremental};
#[cfg(feature = "test-query-execution-observer")]
pub use observed::partitioned_computation_runs_on_this_thread_for_test;
pub(super) use recording::{observe_unretained, FullRecording};
pub use retained::WorthQueryPartitionedComputationFullCause;
#[cfg(any(test, feature = "test-query-execution-observer"))]
pub use retained::WorthQueryPartitionedComputationRun;
pub(in crate::domain_computation::primary_graph) use retained::{
    ComputationDeposit, ComputationInstallation,
};
pub(in crate::domain_computation) use retained::{
    ComputationPrior, RetainedComputation, SealedComputationRun,
};

use self::retained::{RetainedBasis, RetainedPartitions, TypedPrior};
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
        basis: Option<RetainedBasisToken>,
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
    membership: Vec<WorthQueryApplicationFactKey>,
    item_keys: BTreeMap<PartitionItemId, Vec<WorthQueryApplicationFactKey>>,
    partitions: BTreeMap<PartitionIdentity, Vec<WorthQueryApplicationFactKey>>,
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
    reader: &mut Reader<'_, '_, Schema, Operation>,
    installation: &ComputationInstallation,
    input_digest: [u8; 32],
    remaining_work: RemainingWork,
) -> Begun<Key, Item, Reduced>
where
    Key: Send + Sync + 'static,
    Item: Send + Sync + 'static,
    Reduced: Send + Sync + 'static,
{
    let Some(prior) = reader.take_computation_prior() else {
        return Begun::Full {
            basis: None,
            cause: WorthQueryPartitionedComputationFullCause::NoPriorRecord,
        };
    };
    let basis = RetainedBasis::new(installation.clone(), prior.edition, input_digest);
    let full = |basis, cause| Begun::Full {
        basis: Some(RetainedBasisToken(basis)),
        cause,
    };
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
        return full(basis, WorthQueryPartitionedComputationFullCause::Evicted);
    }
    // The installation fixes the state's type, so its own owner's state
    // always downcasts.
    let Ok(typed) =
        Arc::clone(&retained.typed).downcast::<RetainedPartitions<Key, Item, Reduced>>()
    else {
        return full(
            basis,
            WorthQueryPartitionedComputationFullCause::NoPriorRecord,
        );
    };
    let mut run = IncrementalRun {
        basis,
        prior: TypedPrior {
            state: Arc::clone(&retained),
            typed,
        },
        membership_moved: false,
        moved_items: BTreeSet::new(),
        marked: BTreeSet::new(),
        membership: Vec::new(),
        item_keys: BTreeMap::new(),
        partitions: BTreeMap::new(),
    };
    for (key, fact, readers) in retained.facts.facts() {
        let movement = match reader.observe_retained(&COMPARATOR, key) {
            Ok(observed) => match FactMovement::between(fact, &observed).movement() {
                Movement::Unmoved => KeyMovement::Unchanged,
                Movement::Moved => KeyMovement::Changed,
            },
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
            let keys = match read {
                ComputationRead::Membership => &mut run.membership,
                ComputationRead::ItemKey(item) => run.item_keys.entry(item).or_default(),
                ComputationRead::Partition(partition) => {
                    run.partitions.entry(partition).or_default()
                }
            };
            keys.push(key.clone());
        }
    }
    Begun::Incremental(run)
}

#[cfg(test)]
mod tests;

/// Enters the facts a carried call read as that call's admitted reads.
fn enter<'key, Schema, Operation>(
    reader: &mut Reader<'_, '_, Schema, Operation>,
    keys: impl IntoIterator<Item = &'key WorthQueryApplicationFactKey>,
    read: ComputationRead,
) {
    for key in keys {
        reader.enter_carried(&COMPARATOR, key.clone(), read);
    }
}
