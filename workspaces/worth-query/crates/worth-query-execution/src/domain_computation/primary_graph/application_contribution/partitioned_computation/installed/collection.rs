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
    pub(in crate::domain_computation::primary_graph) fn with_collection_check<Completeness>(
        mut self,
    ) -> Self
    where
        Completeness:
            ApplicationCollectionCompleteness<Value = Owner::Output, Stopped = Owner::Stopped>,
    {
        self.completion = Some(Completeness::check);
        self
    }
}
