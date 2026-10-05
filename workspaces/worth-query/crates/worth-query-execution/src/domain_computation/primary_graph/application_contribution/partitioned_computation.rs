//! The partitioned owner binding of a `DeterministicPartitioned` computation.
//!
//! The owner names the items of an input, keys each one, gathers one
//! partition's data and computes one partition at a time, then reduces the
//! results. It reads the input through the reader of the handler that runs
//! the computation, so Query knows which fact each owner call read. Query
//! derives each partition's identity from its key, runs the partitions
//! through `worth-execution`'s map under the computation's declared work
//! ceiling and reduces them over the canonical tree.
//!
//! When a producer runs the computation, the run leaves its state on the
//! record its attempt published, and the producer's next run gathers and
//! computes again only the partitions whose facts changed, under the same
//! input value, membership and keys. Every other run recomputes every
//! partition, for a cause the test observer is shown.

mod compute;
mod denial;
mod incremental;
mod installed;
mod plan;
mod reader;
mod remaining_work;
mod routing;

pub use compute::{
    WorthQueryCompletedPartitionedComputation, WorthQueryPreparedPartitionedComputation,
};
pub use denial::{WorthQueryComputationPartitionStop, WorthQueryPartitionedComputationDenial};
pub use incremental::WorthQueryPartitionedComputationFullCause;
#[cfg(feature = "test-query-execution-observer")]
pub use incremental::{
    partitioned_computation_runs_on_this_thread_for_test, WorthQueryPartitionedComputationRun,
};
pub(in crate::domain_computation::primary_graph) use incremental::{
    Comparator, ComputationDeposit,
};
pub(in crate::domain_computation) use incremental::{
    ComputationPrior, RetainedComputation, SealedComputationRun,
};
pub(in crate::domain_computation::primary_graph) use installed::ComputationRetention;
pub use installed::WorthQueryInstalledPartitionedComputation;
pub use plan::{
    WorthQueryComputationPartitionMembers, WorthQueryComputationPartitionPlan,
    WorthQueryComputationPartitionView,
};
pub use reader::{
    WorthQueryComputationInputDenial, WorthQueryComputationReadDenial, WorthQueryComputationReader,
};

use worth_execution::{CanonicalBits, ChargedBytes};
use worth_query_declaration::facade::application_program::{
    ApplicationComputationInput, ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial};

/// The value a computation's input binding carries.
type InputValue<Schema, Feature, Computation> = <<Computation as ApplicationManagedComputation<
    Schema,
    Feature,
>>::Input as ApplicationComputationInput>::Value;

/// Owns a `DeterministicPartitioned` computation.
///
/// `partitions`, `partition_key`, `gather` and `complete` run on the owner
/// thread. `partitions` names the input's items, `partition_key` runs once per
/// item and `gather` once per partition, in ascending item and partition
/// identity order. Each is lent the handler's reader, and the facts it reads
/// are recorded as the membership's, that item's key's or that partition's.
/// `compute_partition` runs once per partition and may run on any worker, so
/// it is handed what `gather` returned and no reader. The reducer's `combine`
/// sees partitions in partition identity order, which is the order of their
/// keys' digests: deterministic, and not chosen by the author.
///
/// # Purity
///
/// Unchanged facts mean an unchanged partition. `partition_key`, `gather` and
/// `compute_partition` are pure in the input value, the partition's key and
/// the facts read through the reader: they read no clock, no global and no
/// state of the owner that can differ between two runs. A partition whose
/// facts did not change is entitled to keep its last result without being
/// gathered or computed again.
pub trait WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>:
    Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
{
    /// The operation whose handler runs the computation. The owner reads what
    /// that operation declares it reads.
    type Operation;
    /// One item of the input as `partitions` hands it to `partition_key` and
    /// `gather`: what the owner reads the item by. A producer's run keeps its
    /// items for the next run, charged at their bytes.
    type Item: Send + Sync + ChargedBytes + 'static;
    /// One partition's data, gathered on the owner thread for its kernel.
    type Gathered: Send + Sync + ChargedBytes;
    /// One partition's result, and the reduced result of them all. Its
    /// canonical bits are what "the same result" means.
    type PartitionResult: Clone + Send + Sync + ChargedBytes + CanonicalBits + 'static;
    type Output;
    type Stopped: Send + ChargedBytes + 'static;

    /// Names the items the input holds. What it reads is a membership fact:
    /// a change to it changes which items there are.
    fn partitions(
        &self,
        reader: &mut WorthQueryComputationReader<'_, '_, '_, Schema, Self::Operation>,
        input: &InputValue<Schema, Feature, Computation>,
    ) -> Result<
        WorthQueryComputationPartitionPlan<Self::Item>,
        WorthQueryComputationInputDenial<Self::Stopped>,
    >;

    /// The key of the partition `item` belongs to. What it reads is that
    /// item's key fact: a change to it can move the item to another partition.
    fn partition_key(
        &self,
        reader: &mut WorthQueryComputationReader<'_, '_, '_, Schema, Self::Operation>,
        input: &InputValue<Schema, Feature, Computation>,
        item: &Self::Item,
    ) -> Result<Computation::Partition, WorthQueryComputationInputDenial<Self::Stopped>>;

    /// Gathers one partition's data, reads across its boundary included. What
    /// it reads is that partition's fact, and a fact two partitions read is
    /// a fact of both.
    fn gather(
        &self,
        reader: &mut WorthQueryComputationReader<'_, '_, '_, Schema, Self::Operation>,
        input: &InputValue<Schema, Feature, Computation>,
        partition: WorthQueryComputationPartitionMembers<'_, Computation::Partition, Self::Item>,
    ) -> Result<Self::Gathered, WorthQueryComputationInputDenial<Self::Stopped>>;

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Computation::Partition, Self::Gathered>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Self::PartitionResult, WorthQueryManagedComputationDenial<Self::Stopped>>;

    fn reducer(&self) -> WorthQueryDeterministicReducer<Self::PartitionResult>;

    fn complete(&self, reduced: Self::PartitionResult) -> Result<Self::Output, Self::Stopped>;
}

/// How partition results become one result.
///
/// The canonical tree fixes the association for a given set of partitions, so
/// a result that depends on it, such as a floating-point sum, is the same bits
/// on every run and at every worker count.
pub struct WorthQueryDeterministicReducer<Reduced> {
    identity: fn() -> Reduced,
    combine: fn(&Reduced, &Reduced) -> Reduced,
}

impl<Reduced> WorthQueryDeterministicReducer<Reduced> {
    /// Reduces over the canonical tree. `identity` is the result of no
    /// partitions.
    pub const fn canonical(
        identity: fn() -> Reduced,
        combine: fn(&Reduced, &Reduced) -> Reduced,
    ) -> Self {
        Self { identity, combine }
    }
}

#[cfg(test)]
mod attribution_tests;
#[cfg(test)]
mod tests;
