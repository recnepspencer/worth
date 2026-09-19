use super::{
    UiInoperableIntentCandidate, UiIntentAffinityPosture, UiIntentOperabilityOutcome,
    UiIntentOperabilityProof,
};

pub(crate) fn evaluate_intent_operability(
    candidate: super::super::payload::UiPreparedIntentPayload,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
) -> UiIntentOperabilityOutcome {
    let basis = candidate.operability_basis();
    let decision = basis.decision(affinity(&candidate, generation, mounted));
    if decision.is_operable() {
        UiIntentOperabilityOutcome::Operable(UiIntentOperabilityProof::new(candidate, decision))
    } else {
        UiIntentOperabilityOutcome::Inoperable(UiInoperableIntentCandidate::new(
            candidate, decision,
        ))
    }
}

fn affinity(
    candidate: &super::super::payload::UiPreparedIntentPayload,
    current: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
) -> UiIntentAffinityPosture {
    let basis = candidate.input_basis();
    if basis.generation().session_identity() != current.session_identity() {
        return UiIntentAffinityPosture::WrongWorld;
    }
    if basis.generation().prepared_generation() != current.prepared_generation()
        || mounted.has_active_presentation_attempt()
        || mounted.view().current_frame() != Some(basis.publication_frame())
    {
        return UiIntentAffinityPosture::RebindRequired;
    }
    if crate::runtime::interaction::targeting::require_current_target(mounted, basis.target())
        .is_err()
    {
        return UiIntentAffinityPosture::Stale;
    }
    UiIntentAffinityPosture::Current
}
