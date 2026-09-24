use super::PendingCompletionContext;
use crate::mounting::presentation::coordinator::surface_uncertainty::PresentationSurfaceUncertainty;
use crate::mounting::presentation::outcome::UiMountedSurfacePresentationReceipt;
use crate::mounting::presentation::presented_surface::{
    UiIssuedSurfacePresentation, UiPresentedSurfaceWitness,
};

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
    authority: &crate::host::adapter::UiHostAdapterSessionAuthority,
    context: &mut PendingCompletionContext<'_, '_>,
) -> Option<PresentationSurfaceUncertainty> {
    let surface = context
        .frame
        .surfaces()
        .iter()
        .find(|surface| surface.requirement().binding() == pending.binding)
        .expect("pending binding belongs to retained prepared frame");
    let witness = match UiPresentedSurfaceWitness::admit(
        completion,
        authority,
        UiIssuedSurfacePresentation {
            attempt: context.attempt,
            requirement: surface.requirement(),
            frame: context.frame.canonical_core().frame(),
            expected_effects: &pending.expected_effects,
        },
    ) {
        Ok(witness) => witness,
        Err(refused) => {
            return Some(PresentationSurfaceUncertainty::semantic(
                pending.binding,
                Some(refused.cost()),
                pending.semantic_receipts.into_vec(),
            ));
        }
    };
    let posture = match super::super::presented_semantic_settlement::settle(
        pending.binding,
        witness.cost(),
        pending.semantic_receipts,
        context.presentation_async.as_deref_mut(),
    ) {
        Ok(posture) => posture,
        Err(uncertainty) => {
            return Some(uncertainty);
        }
    };
    context.progress.superseded |= posture
        == super::super::presented_semantic_settlement::PresentedSemanticPosture::Superseded;
    context
        .progress
        .completed
        .push(UiMountedSurfacePresentationReceipt::new(witness));
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
