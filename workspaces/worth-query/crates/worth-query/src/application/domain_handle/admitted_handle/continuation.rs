use crate::application::{
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
};
use crate::binding_pipeline::{
    WorthQueryContinuationBindingRequest, WorthQueryResolveContinuationFromTargetRequest,
};
use crate::continuation_pipeline::{
    execute_prepared_continuation_on_handle, ordinary_outcome_from_continuation_checked,
    ordinary_outcome_from_execution_checked, prepare_continuation_from_context_on_handle,
    prepare_continuation_from_target_on_handle, WorthQueryContinuationExecution,
    WorthQueryContinuationExecutionChecked, WorthQueryContinuationExecutionOutcome,
    WorthQueryContinuationExecutionTranscript, WorthQueryExecutePreparedContinuationRequest,
    WorthQueryPreparedContinuation, WorthQueryPreparedContinuationChecked,
    WorthQueryPreparedContinuationOutcome, WorthQueryPreparedContinuationTranscript,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;

use super::WorthQueryInstalledDomainDeclarationContext;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn prepare_continuation_from_target<I>(
        &self,
        request: WorthQueryResolveContinuationFromTargetRequest<D, I>,
    ) -> Result<WorthQueryPreparedContinuationOutcome<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(prepare_continuation_from_target_on_handle(self, request)?
            .into_checked()
            .into_outcome())
    }

    pub fn prepare_continuation_from_target_outcome<I>(
        &self,
        request: WorthQueryResolveContinuationFromTargetRequest<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQueryPreparedContinuation<D, I>>,
        IdentityDerivationDenial,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(ordinary_outcome_from_continuation_checked(
            prepare_continuation_from_target_on_handle(self, request)?.into_checked(),
        ))
    }

    pub fn prepare_continuation_from_target_checked<I>(
        &self,
        request: WorthQueryResolveContinuationFromTargetRequest<D, I>,
    ) -> Result<WorthQueryPreparedContinuationChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(prepare_continuation_from_target_on_handle(self, request)?.into_checked())
    }

    pub fn prepare_continuation_from_target_proof<I>(
        &self,
        request: WorthQueryResolveContinuationFromTargetRequest<D, I>,
    ) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        prepare_continuation_from_target_on_handle(self, request)
    }

    pub fn prepare_continuation_from_context<I>(
        &self,
        request: WorthQueryContinuationBindingRequest<D, I>,
    ) -> Result<WorthQueryPreparedContinuationOutcome<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(prepare_continuation_from_context_on_handle(self, request)?
            .into_checked()
            .into_outcome())
    }

    pub fn prepare_continuation_from_context_outcome<I>(
        &self,
        request: WorthQueryContinuationBindingRequest<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQueryPreparedContinuation<D, I>>,
        IdentityDerivationDenial,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(ordinary_outcome_from_continuation_checked(
            prepare_continuation_from_context_on_handle(self, request)?.into_checked(),
        ))
    }

    pub fn prepare_continuation_from_context_checked<I>(
        &self,
        request: WorthQueryContinuationBindingRequest<D, I>,
    ) -> Result<WorthQueryPreparedContinuationChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(prepare_continuation_from_context_on_handle(self, request)?.into_checked())
    }

    pub fn prepare_continuation_from_context_proof<I>(
        &self,
        request: WorthQueryContinuationBindingRequest<D, I>,
    ) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        prepare_continuation_from_context_on_handle(self, request)
    }

    pub fn execute_prepared_continuation<I>(
        &self,
        prepared: WorthQueryPreparedContinuation<D, I>,
    ) -> Result<WorthQueryContinuationExecutionOutcome<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(execute_prepared_continuation_on_handle(
            self,
            WorthQueryExecutePreparedContinuationRequest::new(prepared),
        )?
        .into_checked()
        .into_outcome())
    }

    pub fn execute_prepared_continuation_outcome<I>(
        &self,
        prepared: WorthQueryPreparedContinuation<D, I>,
    ) -> Result<
        WorthQueryOrdinaryOutcome<WorthQueryContinuationExecution<D, I>>,
        IdentityDerivationDenial,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(ordinary_outcome_from_execution_checked(
            execute_prepared_continuation_on_handle(
                self,
                WorthQueryExecutePreparedContinuationRequest::new(prepared),
            )?
            .into_checked(),
        ))
    }

    pub fn execute_prepared_continuation_checked<I>(
        &self,
        prepared: WorthQueryPreparedContinuation<D, I>,
    ) -> Result<WorthQueryContinuationExecutionChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(execute_prepared_continuation_on_handle(
            self,
            WorthQueryExecutePreparedContinuationRequest::new(prepared),
        )?
        .into_checked())
    }

    pub fn execute_prepared_continuation_proof<I>(
        &self,
        prepared: WorthQueryPreparedContinuation<D, I>,
    ) -> Result<WorthQueryContinuationExecutionTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        execute_prepared_continuation_on_handle(
            self,
            WorthQueryExecutePreparedContinuationRequest::new(prepared),
        )
    }
}
