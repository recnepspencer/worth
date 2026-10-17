//! Checked binding of a declared collection to its existing keyed computation.
use super::super::{
    WorthQueryInstalledPartitionedComputation, WorthQueryPartitionedComputationOwner,
};
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationCollectionCompleteness, ApplicationDerivedCollection, ApplicationFeature,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationContributionSetup<'_, Schema> {
    /// Installs the collection's keyed owner with its declared pure completeness
    /// check. Ordinary partitioned installation alone cannot satisfy a collection.
    /// The check runs on the actual completed value before publication effects.
    pub fn derived_collection<Feature, Collection, Owner>(
        &mut self,
        owner: Owner,
    ) -> Result<
        WorthQueryInstalledPartitionedComputation<Schema, Feature, Collection::Computation, Owner>,
        WorthQueryPrimaryGraphInstallationDenial,
    >
    where
        Feature: ApplicationFeature<Schema>,
        Collection: ApplicationDerivedCollection<Schema, Feature>,
        Owner: WorthQueryPartitionedComputationOwner<
            Schema,
            Feature,
            Collection::Computation,
            Output = <Collection::Completeness as ApplicationCollectionCompleteness>::Value,
            Stopped = <Collection::Completeness as ApplicationCollectionCompleteness>::Stopped,
        >,
    {
        let installed =
            self.partitioned_computation::<Feature, Collection::Computation, Owner>(owner)?;
        self.handlers.bind_collection::<Feature, Collection>();
        Ok(installed.with_collection_check::<Collection::Completeness>())
    }
}
