use worth_ui_host_contract::{
    UiHostPresentationCompletionToken, UiHostSurfaceCancellationOutcome,
    UiHostSurfaceInFlightCompletion, UiHostSurfacePresentationOutcome,
    UiMountedFrameConsumptionView,
};

use crate::native::{
    host_state::{UiNativePendingTextContinuation, UiNativePendingTextPresentation},
    UiNativeHostState,
};

pub(super) enum UiMountedTextWorkOutcome {
    Ready,
    Pending(
        (
            worth_ui_host_contract::UiGlyphRasterTransactionPending,
            Box<[worth_ui_host_contract::UiGlyphRasterPinRequest]>,
        ),
    ),
    Terminal(UiHostSurfacePresentationOutcome),
}

pub(super) fn begin(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
) -> UiMountedTextWorkOutcome {
    let Some(work) = view.text_raster_work() else {
        return UiMountedTextWorkOutcome::Ready;
    };
    if !super::presentation::glyph_run_admission::admits(view, work) {
        return UiMountedTextWorkOutcome::Terminal(super::presentation::adapter_declined());
    }
    let live_pins = state.text_atlas.pin_observations();
    let reconstruction_additions;
    let reconstruction_releases;
    let pins = if matches!(
        view.presentation_work(),
        worth_ui_host_contract::UiMountedPresentationWorkView::Reconstruction(_)
    ) {
        reconstruction_additions = work
            .pins()
            .additions()
            .iter()
            .copied()
            .filter(|pin| !live_pins.iter().any(|live| live.matches(*pin)))
            .collect::<Vec<_>>();
        reconstruction_releases = work
            .pins()
            .releases()
            .iter()
            .copied()
            .filter(|pin| live_pins.iter().any(|live| live.matches(*pin)))
            .collect::<Vec<_>>();
        worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(
            &reconstruction_additions,
            &reconstruction_releases,
        )
    } else {
        work.pins()
    };
    let mut rasterizer = CallbackRasterizer(work);
    let outcome = super::text_atlas::perform(
        state,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::from_view(view),
        work.demands(),
        pins,
        &mut rasterizer,
    );
    state.record_compiler_total_peak();
    match outcome {
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome::Committed(_) => {
            state.text_pins_by_binding.insert(
                view.binding().diagnostic_value(),
                work.binding_pins().to_vec().into_boxed_slice(),
            );
            UiMountedTextWorkOutcome::Ready
        }
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome::Pending(pending) => {
            UiMountedTextWorkOutcome::Pending((
                pending,
                work.binding_pins().to_vec().into_boxed_slice(),
            ))
        }
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome::RejectedBeforeEffects(_) => {
            UiMountedTextWorkOutcome::Terminal(super::presentation::adapter_declined())
        }
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome::RejectedAfterRasterization(_) => {
            UiMountedTextWorkOutcome::Terminal(
                super::presentation::mark_presentation_indeterminate(state),
            )
        }
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome::EffectsIndeterminate(_) => {
            UiMountedTextWorkOutcome::Terminal(
                super::presentation::mark_presentation_indeterminate(state),
            )
        }
    }
}

pub(super) fn retain_pending(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    token: &UiHostPresentationCompletionToken,
    (atlas, binding_pins): (
        worth_ui_host_contract::UiGlyphRasterTransactionPending,
        Box<[worth_ui_host_contract::UiGlyphRasterPinRequest]>,
    ),
    continuation: UiNativePendingTextContinuation,
) {
    state.pending_text_presentations.insert(
        token.diagnostic_value(),
        UiNativePendingTextPresentation {
            atlas,
            continuation,
            binding: view.binding().diagnostic_value(),
            binding_pins,
        },
    );
}

pub(super) fn settle_deferred(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    pending: Option<(
        worth_ui_host_contract::UiGlyphRasterTransactionPending,
        Box<[worth_ui_host_contract::UiGlyphRasterPinRequest]>,
    )>,
) -> UiHostSurfacePresentationOutcome {
    let Some(pending) = pending else {
        return UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
        );
    };
    let token = view.issue_text_atlas_completion_token();
    retain_pending(
        state,
        view,
        &token,
        pending,
        UiNativePendingTextContinuation::AtlasReady,
    );
    UiHostSurfacePresentationOutcome::InFlight(token)
}

pub(super) fn complete(
    state: &mut UiNativeHostState,
    token: UiHostPresentationCompletionToken,
) -> UiHostSurfaceInFlightCompletion {
    let key = token.diagnostic_value();
    let Some(mut pending) = state.pending_text_presentations.remove(&key) else {
        return UiHostSurfaceInFlightCompletion::RejectedBeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::AdapterDeclined,
        );
    };
    let outcome = state.complete_pending_text_atlas(pending.atlas);

    state.record_compiler_total_peak();
    match outcome {
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome::Pending(next) => {
            pending.atlas = next;
            state.pending_text_presentations.insert(key, pending);
            UiHostSurfaceInFlightCompletion::Pending(token)
        }
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome::Committed(_) => {
            state
                .text_pins_by_binding
                .insert(pending.binding, pending.binding_pins);
            state.record_text_pin_frame_observation();
            match pending.continuation {
                UiNativePendingTextContinuation::AtlasReady => {
                    UiHostSurfaceInFlightCompletion::RejectedBeforeEffects(
                        worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
                    )
                }
            }
        }
        worth_ui_host_contract::UiGlyphRasterTransactionOutcome::RejectedBeforeEffects(_)
        | worth_ui_host_contract::UiGlyphRasterTransactionOutcome::RejectedAfterRasterization(_)
        | worth_ui_host_contract::UiGlyphRasterTransactionOutcome::EffectsIndeterminate(_) => {
            state.lifecycle.record_presentation_indeterminate();
            UiHostSurfaceInFlightCompletion::PresentationIndeterminate
        }
    }
}

pub(super) fn stop(
    state: &mut UiNativeHostState,
    token: UiHostPresentationCompletionToken,
    reason: worth_ui_host_contract::UiHostSurfaceStopReason,
) -> UiHostSurfaceCancellationOutcome {
    let Some(pending) = state
        .pending_text_presentations
        .remove(&token.diagnostic_value())
    else {
        return UiHostSurfaceCancellationOutcome::CancelledBeforeEffects;
    };
    let _ = match reason {
        worth_ui_host_contract::UiHostSurfaceStopReason::Cancelled => {
            state.cancel_pending_text_atlas(pending.atlas)
        }
        worth_ui_host_contract::UiHostSurfaceStopReason::Superseded => {
            state.supersede_pending_text_atlas(pending.atlas)
        }
    };
    UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun
}

struct CallbackRasterizer<'work>(&'work worth_ui_host_contract::UiMountedTextRasterWork<'work>);

impl worth_ui_host_contract::UiGlyphRasterMissRasterizer for CallbackRasterizer<'_> {
    fn rasterize(
        &mut self,
        misses: worth_ui_host_contract::UiGlyphRasterMissSelectionView<'_>,
        sink: &mut dyn worth_ui_host_contract::UiGlyphRasterBatchSink,
    ) -> Result<(), worth_ui_host_contract::UiGlyphRasterCallbackDenial> {
        self.0.rasterize(misses, sink)
    }
}
