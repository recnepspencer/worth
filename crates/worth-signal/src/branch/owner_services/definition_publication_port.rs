use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdvanceCompletion, SignalBranchAdvanceDenial,
};
use crate::data::error::SignalError;
use crate::logic::transaction::SignalTransaction;

use super::{
    SignalBranchMutationPort, SignalConditionalDefinitionPublicationOperation,
    SignalOwnerCancellationToken,
};

/// The non-cloneable Signal capability reserved for Runtime World definition publication.
/// Ordinary component service bundles never contain this port.
pub struct SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    mutation: SignalBranchMutationPort<D, I, E, Ctx, T>,
}

/// A publication operation admitted against the exact Signal predecessor before
/// either component owner is allowed to move.
pub struct AdmittedSignalConditionalDefinitionPublication {
    operation: SignalConditionalDefinitionPublicationOperation,
}

impl<D, I, E, Ctx, T> SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(mutation: SignalBranchMutationPort<D, I, E, Ctx, T>) -> Self {
        Self { mutation }
    }

    pub fn admit(
        &self,
        operation: SignalConditionalDefinitionPublicationOperation,
        expected: &AdmittedSignalBranchBasis,
    ) -> Result<AdmittedSignalConditionalDefinitionPublication, SignalBranchAdvanceDenial> {
        self.mutation
            .validate_conditional_definition_publication(&operation, expected)?;
        Ok(AdmittedSignalConditionalDefinitionPublication { operation })
    }

    pub fn advance_exact_with_completion<F>(
        &self,
        admitted: AdmittedSignalConditionalDefinitionPublication,
        expected: &AdmittedSignalBranchBasis,
        runtime_ctx: &mut Ctx,
        cancellation: &SignalOwnerCancellationToken,
        apply: F,
    ) -> SignalBranchAdvanceCompletion
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        self.mutation
            .advance_conditional_definition_exact_with_completion(
                admitted.operation,
                expected,
                runtime_ctx,
                cancellation,
                apply,
            )
    }
}
