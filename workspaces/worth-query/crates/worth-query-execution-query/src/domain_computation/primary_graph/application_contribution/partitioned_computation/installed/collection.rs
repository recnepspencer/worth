use super::*;
use worth_query_declaration::facade::application_program::ApplicationCollectionCompleteness;

impl<Schema, Feature, Computation, Owner>
    WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    pub(crate) fn with_collection_check<Completeness>(mut self) -> Self
    where
        Completeness:
            ApplicationCollectionCompleteness<Value = Owner::Output, Stopped = Owner::Stopped>,
    {
        self.completion = Some(Completeness::check);
        self
    }
}

/// Crate-boundary internal port for attaching a collection completeness check
/// during installation. Never exported on `worth-query-execution::facade` or Host.
pub trait InstalledPartitionedComputationInternalPort<Schema, Feature, Computation, Owner>:
    Sized
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    fn with_collection_check<Completeness>(self) -> Self
    where
        Completeness:
            ApplicationCollectionCompleteness<Value = Owner::Output, Stopped = Owner::Stopped>;
}

impl<Schema, Feature, Computation, Owner>
    InstalledPartitionedComputationInternalPort<Schema, Feature, Computation, Owner>
    for WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    fn with_collection_check<Completeness>(self) -> Self
    where
        Completeness:
            ApplicationCollectionCompleteness<Value = Owner::Output, Stopped = Owner::Stopped>,
    {
        Self::with_collection_check::<Completeness>(self)
    }
}
