use crate::application::{
    WorthQueryAdmittedDeclarationProgression,
    WorthQueryDeclarationEntryOrchestrationArtifactPolicy,
    WorthQueryDeclarationEntryOrchestrationExposureLevel,
    WorthQueryDeclarationEntryOrchestrationProduct, WorthQueryDeclarationEnvelope,
    WorthQueryDeclarationEnvelopeChecked, WorthQueryDeclarationEnvelopeTerminalError,
    WorthQueryDeclarationInput, WorthQueryDeclarationRouteIntent, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::common::{envelope_orchestration_identity, envelope_terminal_from_checked};
use super::transcript::WorthQueryDeclarationEnvelopeOrchestrationTranscript;
use crate::application::{
    worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle,
    WorthQueryDeclarationEntryProductChecked,
};

pub(crate) fn worth_query_declaration_envelope_orchestration_from_progressed_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEnvelopeTerminalError<D, I>> {
    match worth_query_checked_declaration_envelope_orchestration_from_progressed_on_handle(
        handle,
        progressed,
        route_intent,
    ) {
        WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => Ok(envelope),
        other => Err(envelope_terminal_from_checked(other)),
    }
}

pub(crate) fn worth_query_checked_declaration_envelope_orchestration_from_progressed_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> Result<WorthQueryDeclarationEnvelopeChecked<D, I>, IdentityDerivationDenial> {
    Ok(
        match worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
            handle,
            progressed,
            WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
            WorthQueryDeclarationEntryOrchestrationArtifactPolicy::CheckedOutcomeOnly,
            WorthQueryDeclarationEntryOrchestrationProduct::Envelope,
            route_intent,
        )?
        .checked
        {
            WorthQueryDeclarationEntryProductChecked::Envelope(checked) => checked,
            _ => panic!("envelope orchestration must project the envelope product"),
        },
    )
}

pub(crate) fn worth_query_declaration_envelope_orchestration_from_progressed_proof_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> Result<WorthQueryDeclarationEnvelopeOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    let lowered =
        worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
            handle,
            progressed,
            WorthQueryDeclarationEntryOrchestrationExposureLevel::ProofVisible,
            WorthQueryDeclarationEntryOrchestrationArtifactPolicy::ProofVisibleTranscript,
            WorthQueryDeclarationEntryOrchestrationProduct::Envelope,
            route_intent,
        )?;
    let checked = match lowered.checked {
        WorthQueryDeclarationEntryProductChecked::Envelope(checked) => checked,
        _ => panic!("envelope orchestration proof must project the envelope product"),
    };
    let outcome_identity = envelope_orchestration_identity(&checked);
    Ok(WorthQueryDeclarationEnvelopeOrchestrationTranscript::new(
        lowered.plan,
        checked,
        lowered.step_records,
        outcome_identity,
    ))
}
