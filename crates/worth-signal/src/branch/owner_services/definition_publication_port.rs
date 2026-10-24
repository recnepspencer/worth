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

    /// Returns the original admitted permit when no canonical movement consumed it.
    /// Secure that custody before inspecting completion or resuming its unwind.
    pub fn advance_exact_with_completion<F>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,

        admitted: AdmittedSignalConditionalDefinitionPublication,
        expected: &AdmittedSignalBranchBasis,
        runtime_ctx: &mut Ctx,
        cancellation: &SignalOwnerCancellationToken,
        apply: F,
    ) -> (
        SignalBranchAdvanceCompletion,
        Option<AdmittedSignalConditionalDefinitionPublication>,
    )
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let mut operation = Some(admitted.operation);
        let completion = self
            .mutation
            .advance_conditional_definition_exact_with_completion(
                execution,
                &mut operation,
                expected,
                runtime_ctx,
                cancellation,
                apply,
            );
        (
            completion,
            operation.map(|operation| AdmittedSignalConditionalDefinitionPublication { operation }),
        )
    }
}
