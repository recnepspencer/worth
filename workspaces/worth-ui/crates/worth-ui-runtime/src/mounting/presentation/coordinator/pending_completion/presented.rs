use super::PendingCompletionContext;
use crate::mounting::presentation::coordinator::surface_uncertainty::PresentationSurfaceUncertainty;
use crate::mounting::presentation::outcome::UiMountedSurfacePresentationReceipt;
use crate::mounting::presentation::terminal::completion_satisfies;

pub(super) struct PresentedPendingSurface {
    pub(super) binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    pub(super) expected_effects: Box<[worth_ui_host_contract::UiMountedEffectFamily]>,
    pub(super) text_candidate: Option<super::super::UiMountedTextPinCandidate>,
    pub(super) semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
    pub(super) text_reuse:
        Option<crate::native_platform::text_presentation::UiMountedTextForegroundReuseUpdate>,
}

pub(super) fn complete(
    pending: PresentedPendingSurface,
    completion: worth_ui_host_contract::UiMountedSurfacePresentationCompletion,
    context: &mut PendingCompletionContext<'_, '_>,
) -> Option<PresentationSurfaceUncertainty> {
    let surface = context
        .frame
        .surfaces()
        .iter()
        .find(|surface| surface.requirement().binding() == pending.binding)
        .expect("pending binding belongs to retained prepared frame");
    if !completion_satisfies(surface, &pending.expected_effects, &completion) {
        return Some(PresentationSurfaceUncertainty::semantic(
            pending.binding,
            Some(completion.cost()),
            pending.semantic_receipts.into_vec(),
        ));
    }
    let posture = match super::super::presented_semantic_settlement::settle(
        pending.binding,
        completion.cost(),
        pending.semantic_receipts,
        context.presentation_async.as_deref_mut(),
    ) {
        Ok(posture) => posture,
        Err(uncertainty) => return Some(uncertainty),
    };
    context.progress.superseded |= posture
        == super::super::presented_semantic_settlement::PresentedSemanticPosture::Superseded;
    let (epoch, effects, adapter_cost) = completion.into_parts();
    context
        .progress
        .completed
        .push(UiMountedSurfacePresentationReceipt::new(
            surface.requirement(),
            epoch,
            effects,
            adapter_cost,
        ));
    if !context.progress.superseded {
        if let Some(candidate) = pending.text_candidate {
            context.text.commit_surface_candidate(candidate);
        }
        if let Some(update) = pending.text_reuse {
            context.text.commit_foreground_reuse(update);
        }
    }
    None
}
