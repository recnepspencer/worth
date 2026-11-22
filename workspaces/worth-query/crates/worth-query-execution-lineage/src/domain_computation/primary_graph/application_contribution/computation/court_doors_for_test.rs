//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Feature, Computation, Owner>
    WorthQueryInstalledManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    #[doc(hidden)]
    pub fn new_for_test(owner: Owner) -> Self {
        Self::new(owner)
    }
}
