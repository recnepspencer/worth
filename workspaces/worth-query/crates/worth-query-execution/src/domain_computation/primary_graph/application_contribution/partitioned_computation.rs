//! The partitioned owner binding of a `DeterministicPartitioned` computation.
//!
//! The owner names the partitions of an input, computes one partition at a
//! time and reduces the results. Query derives each partition's identity from
//! its key, runs the partitions through `worth-execution`'s map under the
//! computation's declared work ceiling and reduces them over the canonical
//! tree. Every run recomputes every partition.

mod denial;
mod installed;
mod plan;
mod routing;

pub use denial::{WorthQueryComputationPartitionStop, WorthQueryPartitionedComputationDenial};
pub use installed::{
    WorthQueryCompletedPartitionedComputation, WorthQueryInstalledPartitionedComputation,
    WorthQueryPreparedPartitionedComputation,
};
pub use plan::{
    WorthQueryComputationPartitionItem, WorthQueryComputationPartitionPlan,
    WorthQueryComputationPartitionView,
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
/// `partitions` and `complete` run on the owner thread. `compute_partition`
/// runs once per partition and may run on any worker, so it reads only its
/// view. The reducer's `combine` sees partitions in partition identity order,
/// which is the order of their keys' digests: deterministic, and not chosen by
/// the author.
pub trait WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>:
    Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
{
    /// One partition's result, and the reduced result of them all. Its
    /// canonical bits are what "the same result" means.
    type PartitionResult: Clone + Send + Sync + ChargedBytes + CanonicalBits + 'static;
    type Output;
    type Stopped: Send + ChargedBytes + 'static;

    fn partitions(
        &self,
        input: &InputValue<Schema, Feature, Computation>,
    ) -> WorthQueryComputationPartitionPlan<Computation::Partition>;

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<
            '_,
            Computation::Partition,
            InputValue<Schema, Feature, Computation>,
        >,
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
mod tests;
