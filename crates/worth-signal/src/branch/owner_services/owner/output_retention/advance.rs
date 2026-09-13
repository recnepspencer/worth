use std::panic::{catch_unwind, AssertUnwindSafe};

use super::SignalAdvanceOutputReservation;
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::owner_services::SignalOwnerCancellationToken;
use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdvanceCompletion, SignalBranchAdvanceOutcome,
};
use crate::data::error::SignalError;
use crate::logic::transaction::SignalTransaction;

impl<D, I, T> SignalAdvanceOutputReservation<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn advance<E, Ctx, F>(
        self,
        expected: &AdmittedSignalBranchBasis,
        runtime_ctx: &mut Ctx,
        cancellation: &SignalOwnerCancellationToken,
        apply: F,
    ) -> SignalBranchAdvanceCompletion
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        self.advance_with_definition_publication(expected, runtime_ctx, cancellation, apply, None)
    }

    pub(in crate::branch::owner_services) fn advance_conditional_definition<E, Ctx, F>(
        self,
        expected: &AdmittedSignalBranchBasis,
        runtime_ctx: &mut Ctx,
        cancellation: &SignalOwnerCancellationToken,
        apply: F,
        publication: crate::branch::owner_services::conditional_execution::SignalConditionalDefinitionPublicationOperation,
    ) -> SignalBranchAdvanceCompletion
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let (scope, mint) = publication.into_advance_parts();
        self.advance_with_definition_publication(
            expected,
            runtime_ctx,
            cancellation,
            apply,
            Some((scope, mint)),
        )
    }

    fn advance_with_definition_publication<E, Ctx, F>(
        self,
        expected: &AdmittedSignalBranchBasis,
        runtime_ctx: &mut Ctx,
        cancellation: &SignalOwnerCancellationToken,
        apply: F,
        definition_publication: Option<
            (
                crate::branch::owner_services::conditional_execution::SignalConditionalDefinitionPublicationScope,
                crate::branch::owner_services::conditional_execution::SignalConditionalDefinitionAdvanceMint,
            ),
        >,
    ) -> SignalBranchAdvanceCompletion
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let mut completed = None;
        let execution = catch_unwind(AssertUnwindSafe(|| {
            self.cell.advance_into(
                self.admission,
                expected,
                runtime_ctx,
                cancellation,
                apply,
                &mut completed,
                definition_publication
                    .as_ref()
                    .map(|(scope, _)| scope.clone()),
            )
        }));
        let Some(completed) = completed else {
            return match execution {
                Ok(Err(denial)) => SignalBranchAdvanceCompletion::returned(Err(denial)),
                Err(payload) => SignalBranchAdvanceCompletion::unwound(None, payload),
                Ok(Ok(())) => unreachable!("successful movement carries exact completion"),
            };
        };
        // The branch guard has returned or unwound. Admit only the carried
        // observation using retention reserved before entering that guard.
        let (branch_id, observation, transaction) = completed.into_output_parts();
        let mut retention = self.retention;
        let basis = self.owner.admit_canonical_basis(
            observation,
            branch_id,
            self.cell.incarnation().get(),
            retention.take_one(),
        );
        let definition_binding = definition_publication.map(|(_, mint)| mint.bind(basis.clone()));
        let outcome =
            SignalBranchAdvanceOutcome::owner_issued(basis, transaction, definition_binding);
        if let Err(payload) = execution {
            return SignalBranchAdvanceCompletion::unwound(Some(outcome), payload);
        }
        match catch_unwind(AssertUnwindSafe(|| {
            self.admission
                .reach_operation_boundary(SignalOwnerOperationBoundary::OutcomeConstruction);
        })) {
            Ok(()) => SignalBranchAdvanceCompletion::returned(Ok(outcome)),
            Err(payload) => SignalBranchAdvanceCompletion::unwound(Some(outcome), payload),
        }
    }
}
