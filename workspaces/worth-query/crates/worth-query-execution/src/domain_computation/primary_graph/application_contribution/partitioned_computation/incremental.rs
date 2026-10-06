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
//! order, membership, item keys, gathers and kernels by identity, then the
//! combines, so a ceiling names the partition a full run would name.

mod next_tree;
mod recording;
mod retained;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) use next_tree::{ComputedIncremental, PreparedIncremental};
pub(super) use recording::{observe_unretained, FullRecording};
#[cfg(feature = "test-query-execution-observer")]
pub use retained::partitioned_computation_runs_on_this_thread_for_test;
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
use super::compute::Denial;
use super::remaining_work::RemainingWork;
use super::{
    InputValue, WorthQueryComputationPartitionMembers, WorthQueryComputationReader,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};
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

/// A comparison that found the membership and every key unchanged.
pub(super) struct IncrementalRun<Key, Item, Reduced> {
    basis: RetainedBasis,
    prior: TypedPrior<Key, Item, Reduced>,
    marked: BTreeSet<PartitionIdentity>,
    membership: Vec<WorthQueryApplicationFactKey>,
    item_keys: BTreeMap<PartitionItemId, Vec<WorthQueryApplicationFactKey>>,
    partitions: BTreeMap<PartitionIdentity, Vec<WorthQueryApplicationFactKey>>,
}

/// Compares the state the producer handed the reader with this attempt's
/// facts. `installation` names the installed owner and `input_digest` is
/// the input value's digest, already charged.
pub(super) fn begin<Key, Item, Reduced, Schema, Operation>(
    reader: &mut Reader<'_, '_, Schema, Operation>,
    installation: &ComputationInstallation,
    input_digest: [u8; 32],
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
        if movement != KeyMovement::Unchanged {
            if readers.partitioner() {
                return full(
                    run.basis,
                    WorthQueryPartitionedComputationFullCause::PartitionerRebuilt,
                );
            }
            run.marked.extend(readers.partitions().iter().copied());
        }
        for read in readers.reads() {
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

impl<Key, Item, Reduced> IncrementalRun<Key, Item, Reduced>
where
    Key: Send + Sync + 'static,
{
    /// Replays the membership, every item's key and every unmarked gathering
    /// from the retained run, and gathers the marked partitions again.
    ///
    /// A carried call charges what it charged, where it charged it, and its
    /// facts and reached entities enter the attempt as its reads. A call the
    /// remaining work would not pass is made instead, and fails as a full run
    /// fails. `remaining_work` is what the digest left of the declared work.
    pub(super) fn prepare<Schema, Feature, Computation, Owner>(
        self,
        owner: &Owner,
        reader: &mut Reader<'_, '_, Schema, Owner::Operation>,
        input: &InputValue<Schema, Feature, Computation>,
        mut remaining_work: RemainingWork,
        declared_bytes: u64,
    ) -> Result<
        PreparedIncremental<Key, Item, Reduced, Owner::Gathered>,
        Denial<Schema, Feature, Computation, Owner>,
    >
    where
        Schema: ApplicationSchema,
        Feature: ApplicationFeature<Schema>,
        Computation: ApplicationManagedComputation<Schema, Feature, Partition = Key>,
        Owner: WorthQueryPartitionedComputationOwner<
            Schema,
            Feature,
            Computation,
            Item = Item,
            PartitionResult = Reduced,
        >,
    {
        let retained = Arc::clone(&self.prior.typed);
        if reader.carry(&COMPARATOR, &retained.membership) {
            enter(reader, &self.membership, ComputationRead::Membership);
        } else {
            reader.attributed(ComputationRead::Membership, |reader| {
                owner.partitions(&mut WorthQueryComputationReader::lend(reader), input)
            })?;
        }
        for (item, call) in retained.item_keys.iter() {
            let read = ComputationRead::ItemKey(*item);
            if reader.carry(&COMPARATOR, &call.charge) {
                enter(reader, self.item_keys.get(item).into_iter().flatten(), read);
            } else {
                reader.attributed(read, |reader| {
                    owner.partition_key(
                        &mut WorthQueryComputationReader::lend(reader),
                        input,
                        &retained.items[item],
                    )
                })?;
            }
            remaining_work
                .spend(Some(call.declared_units))
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        }
        let mut prepared =
            PreparedIncremental::new(remaining_work, declared_bytes, self.basis, self.prior);
        for (identity, partition) in &retained.partitions {
            let read = ComputationRead::Partition(*identity);
            if !self.marked.contains(identity) && reader.carry(&COMPARATOR, &partition.gather) {
                enter(
                    reader,
                    self.partitions.get(identity).into_iter().flatten(),
                    read,
                );
                continue;
            }
            let (gathered, charge) = reader.measured(read, |reader| {
                owner.gather(
                    &mut WorthQueryComputationReader::lend(reader),
                    input,
                    WorthQueryComputationPartitionMembers::new(
                        *identity,
                        &partition.key,
                        &partition.members,
                        &retained.items,
                    ),
                )
            });
            let gathered = gathered.map_err(|denial| {
                WorthQueryPartitionedComputationDenial::gathering(*identity, denial)
            })?;
            prepared.gathered(*identity, partition, gathered, charge)?;
        }
        Ok(prepared)
    }
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
