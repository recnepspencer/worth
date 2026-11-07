use crate::application::{
    WorthQueryAdmittedDeclarationProgression, WorthQueryDeclarationBridgeContinuationMode,
    WorthQueryDeclarationBridgeContinuationRequest, WorthQueryDeclarationBridgeTruthContext,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput,
    WorthQueryDeclarationSignalCompatibilityInput, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;
use crate::signal_compatibility_orchestration::{
    WorthQuerySignalCompatibilityOrchestration, WorthQuerySignalCompatibilityOrchestrationChecked,
    WorthQuerySignalCompatibilityOrchestrationInput,
    WorthQuerySignalCompatibilityOrchestrationOutcome,
    WorthQuerySignalCompatibilityOrchestrationTranscript,
};

pub(super) fn prepare_preview_for_active_face_selection<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial> {
    Ok(handle.orchestrate_signal_compatibility(continuation_input(
        handle,
        progression,
        preview_session_request(),
    )?)?)
}

pub(super) fn prepare_preview_for_active_face_selection_outcome<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<
    WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
    IdentityDerivationDenial,
> {
    Ok(
        handle.orchestrate_signal_compatibility_outcome(continuation_input(
            handle,
            progression,
            preview_session_request(),
        )?)?,
    )
}

pub(super) fn prepare_preview_for_active_face_selection_checked<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial> {
    Ok(
        handle.orchestrate_signal_compatibility_checked(continuation_input(
            handle,
            progression,
            preview_session_request(),
        )?)?,
    )
}

pub(super) fn prepare_preview_for_active_face_selection_proof<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    Ok(
        handle.orchestrate_signal_compatibility_proof(continuation_input(
            handle,
            progression,
            preview_session_request(),
        )?)?,
    )
}

pub(super) fn prepare_runtime_route_for_active_face_selection<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial> {
    Ok(handle.orchestrate_signal_compatibility(continuation_input(
        handle,
        progression,
        runtime_route_request(),
    )?)?)
}

pub(super) fn prepare_runtime_route_for_active_face_selection_outcome<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<
    WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
    IdentityDerivationDenial,
> {
    Ok(
        handle.orchestrate_signal_compatibility_outcome(continuation_input(
            handle,
            progression,
            runtime_route_request(),
        )?)?,
    )
}

pub(super) fn prepare_runtime_route_for_active_face_selection_checked<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial> {
    Ok(
        handle.orchestrate_signal_compatibility_checked(continuation_input(
            handle,
            progression,
            runtime_route_request(),
        )?)?,
    )
}

pub(super) fn prepare_runtime_route_for_active_face_selection_proof<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    Ok(
        handle.orchestrate_signal_compatibility_proof(continuation_input(
            handle,
            progression,
            runtime_route_request(),
        )?)?,
    )
}

pub(super) fn prepare_current_truth_view_for_active_face_selection<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial> {
    Ok(handle.orchestrate_signal_compatibility(continuation_input(
        handle,
        progression,
        truth_view_request(WorthQueryDeclarationBridgeTruthContext::Current),
    )?)?)
}

pub(super) fn prepare_current_truth_view_for_active_face_selection_outcome<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<
    WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
    IdentityDerivationDenial,
> {
    Ok(
        handle.orchestrate_signal_compatibility_outcome(continuation_input(
            handle,
            progression,
            truth_view_request(WorthQueryDeclarationBridgeTruthContext::Current),
        )?)?,
    )
}

pub(super) fn prepare_current_truth_view_for_active_face_selection_checked<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial> {
    Ok(
        handle.orchestrate_signal_compatibility_checked(continuation_input(
            handle,
            progression,
            truth_view_request(WorthQueryDeclarationBridgeTruthContext::Current),
        )?)?,
    )
}

pub(super) fn prepare_current_truth_view_for_active_face_selection_proof<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    Ok(
        handle.orchestrate_signal_compatibility_proof(continuation_input(
            handle,
            progression,
            truth_view_request(WorthQueryDeclarationBridgeTruthContext::Current),
        )?)?,
    )
}

pub(super) fn prepare_historical_truth_view_for_active_face_selection<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationOutcome<D, I>, IdentityDerivationDenial> {
    Ok(handle.orchestrate_signal_compatibility(continuation_input(
        handle,
        progression,
        truth_view_request(WorthQueryDeclarationBridgeTruthContext::Historical),
    )?)?)
}

pub(super) fn prepare_historical_truth_view_for_active_face_selection_outcome<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<
    WorthQueryOrdinaryOutcome<WorthQuerySignalCompatibilityOrchestration<D, I>>,
    IdentityDerivationDenial,
> {
    Ok(
        handle.orchestrate_signal_compatibility_outcome(continuation_input(
            handle,
            progression,
            truth_view_request(WorthQueryDeclarationBridgeTruthContext::Historical),
        )?)?,
    )
}

pub(super) fn prepare_historical_truth_view_for_active_face_selection_checked<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationChecked<D, I>, IdentityDerivationDenial> {
    Ok(
        handle.orchestrate_signal_compatibility_checked(continuation_input(
            handle,
            progression,
            truth_view_request(WorthQueryDeclarationBridgeTruthContext::Historical),
        )?)?,
    )
}

pub(super) fn prepare_historical_truth_view_for_active_face_selection_proof<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    Ok(
        handle.orchestrate_signal_compatibility_proof(continuation_input(
            handle,
            progression,
            truth_view_request(WorthQueryDeclarationBridgeTruthContext::Historical),
        )?)?,
    )
}

fn continuation_input<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    handle: &WorthQueryInstalledDomainDeclarationContext<
        D,
        impl WorthQueryDomainOperatingContext<D>,
    >,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
    bridge_request: WorthQueryDeclarationBridgeContinuationRequest,
) -> Result<WorthQuerySignalCompatibilityOrchestrationInput<D, I>, IdentityDerivationDenial> {
    Ok(
        WorthQuerySignalCompatibilityOrchestrationInput::new(signal_subject_from_progressed(
            handle,
            progression,
        )?)
        .with_required_aspect_contract(I::Family::aspect_contract())
        .with_bridge_request(bridge_request),
    )
}

fn signal_subject_from_progressed<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<
        D,
        impl WorthQueryDomainOperatingContext<D>,
    >,
    progression: WorthQueryAdmittedDeclarationProgression<D, I>,
) -> Result<WorthQueryDeclarationSignalCompatibilityInput<D, I>, IdentityDerivationDenial> {
    let envelope_checked = handle.orchestrate_envelope_from_progressed_checked(progression);
    Ok(WorthQueryDeclarationSignalCompatibilityInput::envelope_checked(envelope_checked))
}

fn preview_session_request() -> WorthQueryDeclarationBridgeContinuationRequest {
    WorthQueryDeclarationBridgeContinuationRequest::new(
        WorthQueryDeclarationBridgeContinuationMode::PreviewSession,
        WorthQueryDeclarationBridgeTruthContext::Preview,
    )
}

fn runtime_route_request() -> WorthQueryDeclarationBridgeContinuationRequest {
    WorthQueryDeclarationBridgeContinuationRequest::new(
        WorthQueryDeclarationBridgeContinuationMode::RuntimeRoute,
        WorthQueryDeclarationBridgeTruthContext::Current,
    )
}

fn truth_view_request(
    truth_context: WorthQueryDeclarationBridgeTruthContext,
) -> WorthQueryDeclarationBridgeContinuationRequest {
    WorthQueryDeclarationBridgeContinuationRequest::new(
        WorthQueryDeclarationBridgeContinuationMode::TruthView,
        truth_context,
    )
}
