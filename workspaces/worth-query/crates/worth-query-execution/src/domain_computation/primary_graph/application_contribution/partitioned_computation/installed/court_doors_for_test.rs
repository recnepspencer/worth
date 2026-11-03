//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Feature, Computation, Owner> WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner> where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>, {
    #[doc(hidden)]
    pub fn owner_for_test(&self) -> &Arc<Owner> {
        &self.owner
    }
}

impl<Schema, Feature, Computation, Owner> WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner> where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>, {
    #[doc(hidden)]
    pub fn prepare_through_for_test(
        &self, reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Owner::Operation,
        >, execution: &QueryRequestExecution<'_>, input: &InputValue<Schema, Feature, Computation>) -> Result<
        WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    > {
        Self::prepare_through(self, reader, execution, input)
    }
}
