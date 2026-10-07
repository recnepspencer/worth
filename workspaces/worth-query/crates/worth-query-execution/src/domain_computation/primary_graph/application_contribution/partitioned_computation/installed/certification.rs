//! Certification enters the ordinary full route with the same admitted reader.
use super::*;
impl<Schema, Feature, Computation, Owner>
    WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    /// Certification models a new installation of the same owner types. Its
    /// allocation identity is new, so inherited state must fail the ordinary
    /// installation comparison even if every fact and result still matches.
    #[doc(hidden)]
    pub fn prepare_after_reinstallation_for_test<Binding>(
        &self,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
        input: &InputValue<Schema, Feature, Computation>,
    ) -> Result<
        WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    >
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Owner::Operation>,
    {
        let mut reinstalled = self.clone();
        reinstalled.installation = ComputationInstallation::new();
        reinstalled.prepare(reader, input)
    }

    /// A fresh computation of the same admitted input and snapshot. The prior
    /// is consumed normally, facts and charges are recorded normally, and the
    /// completed state uses the same publication custody. Only reuse is off.
    #[doc(hidden)]
    pub fn prepare_without_reuse_for_test<Binding>(
        &self,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
        input: &InputValue<Schema, Feature, Computation>,
    ) -> Result<
        WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    >
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Owner::Operation>,
    {
        super::super::certification_reuse::without_reuse(|| self.prepare(reader, input))
    }
}
