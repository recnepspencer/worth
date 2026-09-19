use worth_ui_host_contract::UiHostSurfaceInFlightCompletion;

use super::super::outcome::UiMountedSurfacePresentationRejection;
use super::super::state::UiPendingMountedSurface;
use super::surface_uncertainty::PresentationSurfaceUncertainty;
use crate::facade::UiHostEffectPort;

#[path = "pending_completion/presented.rs"]
mod presented;

pub(super) struct PendingCompletionContext<'frame, 'state> {
    frame: &'frame super::super::super::UiPreparedMountedFrame,
    progress: &'state mut super::UiMountedPresentationProgress,
    text: &'state mut crate::native_platform::text_presentation::UiNativeMountedTextCoordinator,
    presentation_async:
        Option<&'state mut crate::native_platform::text_presentation::UiPresentationAsyncRuntime>,
}

impl<'frame, 'state> PendingCompletionContext<'frame, 'state> {
    pub(super) fn new(
        frame: &'frame super::super::super::UiPreparedMountedFrame,
        progress: &'state mut super::UiMountedPresentationProgress,
        text: &'state mut crate::native_platform::text_presentation::UiNativeMountedTextCoordinator,
        presentation_async: Option<
            &'state mut crate::native_platform::text_presentation::UiPresentationAsyncRuntime,
        >,
    ) -> Self {
        Self {
            frame,
            progress,
            text,
            presentation_async,
        }
    }
}

pub(super) fn observe_pending_surface(
    host: UiHostEffectPort<'_>,
    pending: UiPendingMountedSurface,
    context: &mut PendingCompletionContext<'_, '_>,
) -> Option<PresentationSurfaceUncertainty> {
    let UiPendingMountedSurface {
        binding,
        token,
        expected_effects,
        text_candidate,
        semantic_receipts,
        text_reuse,
    } = pending;
    match host
        .adapter()
        .complete_mounted_surface(host.authority(), token)
    {
        UiHostSurfaceInFlightCompletion::Pending(token) => {
            context.progress.pending.push(UiPendingMountedSurface {
                binding,
                token,
                expected_effects,
                text_candidate,
                semantic_receipts,
                text_reuse,
            });
            None
        }
        UiHostSurfaceInFlightCompletion::RejectedBeforeEffects(denial) => {
            if denial
                == worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred
            {
                if let Some(candidate) = text_candidate {
                    context.text.commit_surface_candidate(candidate);
                }
            }
            if let Some(owner) = context.presentation_async.as_deref_mut() {
                if semantic_receipts
                    .iter()
                    .any(|receipt| owner.reject_recovery_before_effects(receipt).is_err())
                {
                    return Some(PresentationSurfaceUncertainty::semantic(
                        binding,
                        None,
                        semantic_receipts.into_vec(),
                    ));
                }
            }
            context
                .progress
                .rejected
                .push(UiMountedSurfacePresentationRejection::new(binding, denial));
            None
        }
        UiHostSurfaceInFlightCompletion::PresentationIndeterminate => Some(
            observe_physical_indeterminate(binding, semantic_receipts, context),
        ),
        UiHostSurfaceInFlightCompletion::Presented(completion) => presented::complete(
            presented::PresentedPendingSurface {
                binding,
                expected_effects,
                text_candidate,
                semantic_receipts,
                text_reuse,
            },
            completion,
            context,
        ),
        UiHostSurfaceInFlightCompletion::Superseded(observation) => {
            complete_superseded_surface(binding, semantic_receipts, observation, context)
        }
    }
}

fn complete_superseded_surface(
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
    observation: worth_ui_host_contract::UiMountedSurfacePresentationSupersession,
    context: &mut PendingCompletionContext<'_, '_>,
) -> Option<PresentationSurfaceUncertainty> {
    let Some(owner) = context.presentation_async.as_deref_mut() else {
        return Some(PresentationSurfaceUncertainty::semantic(
            binding,
            Some(observation.cost()),
            semantic_receipts.into_vec(),
        ));
    };
    let payload_byte_len = std::mem::size_of_val(&observation) as u64;
    let mut remaining = std::collections::VecDeque::from(semantic_receipts.into_vec());
    while let Some(receipt) = remaining.pop_front() {
        let worth_ui_query_binding::WorthUiPresentationRecoveryReceipt::Pending(receipt) = receipt
        else {
            remaining.push_front(receipt);
            return Some(PresentationSurfaceUncertainty::semantic(
                binding,
                Some(observation.cost()),
                remaining.into_iter().collect(),
            ));
        };
        if let Err((receipt, _)) =
            owner.admit_superseded_after_physical_observation(receipt, payload_byte_len)
        {
            remaining.push_front(receipt.into());
            return Some(PresentationSurfaceUncertainty::semantic(
                binding,
                Some(observation.cost()),
                remaining.into_iter().collect(),
            ));
        }
    }
    context.progress.superseded = true;
    context.progress.superseded_costs.push(observation.cost());
    None
}

fn observe_physical_indeterminate(
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
    context: &mut PendingCompletionContext<'_, '_>,
) -> PresentationSurfaceUncertainty {
    PresentationSurfaceUncertainty::effects_indeterminate(
        binding,
        None,
        semantic_receipts,
        context.presentation_async.as_deref_mut(),
        true,
    )
}
