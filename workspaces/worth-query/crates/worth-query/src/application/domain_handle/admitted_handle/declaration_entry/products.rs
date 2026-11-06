use crate::application::{
    worth_query_checked_declaration_envelope_orchestration_from_progressed_on_handle,
    worth_query_checked_declaration_receipt_orchestration_from_progressed_on_handle,
    worth_query_checked_declaration_route_orchestration_from_progressed_on_handle,
    worth_query_declaration_envelope_orchestration_from_progressed_on_handle,
    worth_query_declaration_envelope_orchestration_from_progressed_proof_on_handle,
    worth_query_declaration_receipt_orchestration_from_progressed_proof_on_handle,
    worth_query_declaration_route_orchestration_from_progressed_proof_on_handle,
    WorthQueryAdmittedDeclarationProgression, WorthQueryDeclarationEnvelope,
    WorthQueryDeclarationEnvelopeChecked, WorthQueryDeclarationEnvelopeOrchestrationTranscript,
    WorthQueryDeclarationEnvelopeTerminalError, WorthQueryDeclarationInput,
    WorthQueryDeclarationReceiptChecked, WorthQueryDeclarationReceiptOrchestrationTranscript,
    WorthQueryDeclarationRouteOrchestrationTranscript, WorthQueryDeclarationRoutePlanChecked,
    WorthQueryDomainEntryMarker,
};
#[cfg(test)]
use crate::application::{
    worth_query_declaration_receipt_orchestration_from_progressed_on_handle,
    worth_query_declaration_route_orchestration_from_progressed_on_handle,
    WorthQueryDeclarationReceipt, WorthQueryDeclarationReceiptTerminalError,
    WorthQueryDeclarationRouteIntent, WorthQueryDeclarationRoutePlan,
    WorthQueryDeclarationRoutePlanTerminalError,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::WorthQueryInstalledDomainDeclarationContext;
use crate::application::WorthQueryDomainOperatingContext;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    #[cfg(test)]
    pub(crate) fn orchestrate_routes_from_progressed<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<
        WorthQueryDeclarationRoutePlan<D, I>,
        WorthQueryDeclarationRoutePlanTerminalError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        worth_query_declaration_route_orchestration_from_progressed_on_handle(
            self, progressed, None,
        )
    }

    pub(crate) fn orchestrate_routes_from_progressed_checked<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQueryDeclarationRoutePlanChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(
            worth_query_checked_declaration_route_orchestration_from_progressed_on_handle(
                self, progressed, None,
            )?,
        )
    }

    pub(crate) fn orchestrate_routes_from_progressed_proof<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQueryDeclarationRouteOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(
            worth_query_declaration_route_orchestration_from_progressed_proof_on_handle(
                self, progressed, None,
            )?,
        )
    }

    #[cfg(test)]
    pub(crate) fn orchestrate_receipt_from_progressed<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQueryDeclarationReceipt<D, I>, WorthQueryDeclarationReceiptTerminalError<D, I>>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        worth_query_declaration_receipt_orchestration_from_progressed_on_handle(
            self, progressed, None,
        )
    }

    pub(crate) fn orchestrate_receipt_from_progressed_checked<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQueryDeclarationReceiptChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(
            worth_query_checked_declaration_receipt_orchestration_from_progressed_on_handle(
                self, progressed, None,
            )?,
        )
    }

    pub(crate) fn orchestrate_receipt_from_progressed_proof<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQueryDeclarationReceiptOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(
            worth_query_declaration_receipt_orchestration_from_progressed_proof_on_handle(
                self, progressed, None,
            )?,
        )
    }

    pub(crate) fn orchestrate_envelope_from_progressed<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEnvelopeTerminalError<D, I>>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        worth_query_declaration_envelope_orchestration_from_progressed_on_handle(
            self, progressed, None,
        )
    }

    #[cfg(test)]
    pub(crate) fn orchestrate_envelope_from_progressed_with_intent<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
        intent: WorthQueryDeclarationRouteIntent,
    ) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEnvelopeTerminalError<D, I>>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        worth_query_declaration_envelope_orchestration_from_progressed_on_handle(
            self,
            progressed,
            Some(intent),
        )
    }

    pub(crate) fn orchestrate_envelope_from_progressed_checked<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQueryDeclarationEnvelopeChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(
            worth_query_checked_declaration_envelope_orchestration_from_progressed_on_handle(
                self, progressed, None,
            )?,
        )
    }

    pub(crate) fn orchestrate_envelope_from_progressed_proof<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    ) -> Result<WorthQueryDeclarationEnvelopeOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(
            worth_query_declaration_envelope_orchestration_from_progressed_proof_on_handle(
                self, progressed, None,
            )?,
        )
    }

    #[cfg(test)]
    pub(crate) fn orchestrate_envelope_from_progressed_proof_with_intent<I>(
        &self,
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
        intent: WorthQueryDeclarationRouteIntent,
    ) -> Result<WorthQueryDeclarationEnvelopeOrchestrationTranscript<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(
            worth_query_declaration_envelope_orchestration_from_progressed_proof_on_handle(
                self,
                progressed,
                Some(intent),
            )?,
        )
    }
}
