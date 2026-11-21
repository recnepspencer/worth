//! Partitioned computation execution, installed owners, and reader, paired with
//! core types and incremental retention from `worth-query-execution-lineage`.

pub use worth_query_execution_lineage::facade::primary_graph::application_contribution::partitioned_computation::{
    discarded_computation_retention_on_this_thread_for_test,
    full_partitioned_computation_preparations_on_this_thread_for_test, observe_discarded,
    observe_published, partitioned_computation_runs_on_this_thread_for_test,
    certification_reuse, items, partitioned_computation_tree_work_on_this_thread_for_test, plan, preparation_work,
    published_partitioned_computations_on_this_thread_for_test, routing, Comparator,
    CompletedComputationRetention, ComputationDeposit, ComputationPrior, NativeComputationReads,
    PriorAbsence, RetainedComputation, SealedComputationRetention, SealedComputationRun,
    Suppression, WorthQueryComputationInputDenial, WorthQueryComputationPartitionMembers,
    WorthQueryComputationPartitionPlan, WorthQueryComputationPartitionStop,
    WorthQueryComputationPartitionView, WorthQueryComputationReadDenial,
    WorthQueryDeterministicReducer, WorthQueryPartitionedComputationDenial,
    WorthQueryPartitionedComputationFullCause, WorthQueryPartitionedComputationRunForTest,
    WorthQueryPartitionedTreeAccounting, WorthQueryPartitionedTreeMetrics,
    WorthQueryPartitionedTreeRebuildCause, WorthQueryPartitionedTreeRun,
    WorthQueryPublishedComputationStateForTest, WorthQueryReductionInputDenial,
};

mod compute;
pub mod incremental {

    use std::sync::Arc;

    use super::preparation_work::PreparationWork;
    use crate::domain_computation::primary_graph::application_attempt::ComputationRead;
    pub use worth_query_execution_lineage::facade::primary_graph::application_contribution::partitioned_computation::incremental::*;
    pub mod next_tree;
    pub mod prepare;
    mod prepare_ranges;
    mod prepare_rekeys;
    pub use next_tree::{ComputedIncremental, PreparedIncremental};
}
pub mod installed;
mod reader;

pub use compute::{
    WorthQueryCompletedPartitionedComputation, WorthQueryPreparedPartitionedComputation,
};
pub use installed::ComputationRetention;
pub use installed::WorthQueryInstalledPartitionedComputation;
pub use reader::WorthQueryComputationReader;

use worth_execution::CanonicalBits;
use worth_query_declaration::facade::application_program::{
    ApplicationComputationInput, ApplicationComputationPartition, ApplicationFeature,
    ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial};

type InputValue<Schema, Feature, Computation> = <<Computation as ApplicationManagedComputation<
    Schema,
    Feature,
>>::Input as ApplicationComputationInput>::Value;

pub trait WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>:
    Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
{
    type Operation;
    type Item: ApplicationComputationPartition;
    type Gathered: Send + Sync;
    type PartitionResult: Clone + Send + Sync + CanonicalBits + 'static;
    type Output;
    type Stopped: Send + 'static;

    fn partitions(
        &self,
        reader: &mut WorthQueryComputationReader<'_, '_, '_, Schema, Self::Operation>,
        input: &InputValue<Schema, Feature, Computation>,
    ) -> Result<
        WorthQueryComputationPartitionPlan<Self::Item>,
        WorthQueryComputationInputDenial<Self::Stopped>,
    >;

    fn partition_key(
        &self,
        reader: &mut WorthQueryComputationReader<'_, '_, '_, Schema, Self::Operation>,
        input: &InputValue<Schema, Feature, Computation>,
        item: &Self::Item,
    ) -> Result<Computation::Partition, WorthQueryComputationInputDenial<Self::Stopped>>;

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
