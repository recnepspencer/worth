use worth_ui_host_contract::{
    UiHostSurfacePresentationOutcome, UiMountedFrameConsumptionView, UiMountedPresentationWorkView,
};

use crate::native::{
    presentation::{
        present_cold_reconstruction, present_delta, present_initial, UiNativePresentationFailure,
        UiNativeReconstructionFailure, UiWgpuNativePresentationPort,
    },
    UiNativeHostState, UiNativePresentationWorkKind, UiNativeRetainedFrameObservation,
};

use super::presentation_text_atlas as text_atlas;

#[path = "presentation/pending_completion.rs"]
mod pending_completion;
pub(super) use pending_completion::{complete_pending, owns_completion, stop_pending};

#[path = "presentation/failure.rs"]
mod failure;
pub(super) use failure::{adapter_declined, mark_presentation_indeterminate};

#[path = "presentation/completion.rs"]
mod completion;
#[path = "presentation/epoch.rs"]
mod epoch;
use completion::completed;
#[cfg(test)]
use epoch::presentation_epoch;
use failure::{before_effects_declined, before_effects_malformed, malformed};

#[path = "presentation/retained_frame.rs"]
mod retained_frame;

#[path = "presentation/sample.rs"]
mod sample;

#[path = "presentation/glyph_run_admission.rs"]
pub(super) mod glyph_run_admission;

#[cfg(test)]
#[path = "presentation/text_atlas_tests.rs"]
pub(crate) mod text_atlas_tests;

pub(super) fn perform_native_presentation(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
) -> UiHostSurfacePresentationOutcome {
    let outcome = perform_native_presentation_work(state, view);
    state
        .lifecycle
        .observe_presentation_retry_outcome(view.attempt(), &outcome);
    outcome
}

fn perform_native_presentation_work(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
) -> UiHostSurfacePresentationOutcome {
    let key = view.binding().diagnostic_value();
    if matches!(
        view.presentation_work(),
        UiMountedPresentationWorkView::Reconstruction(_)
    ) && state.lifecycle.recovery_required(key)
        && !crate::native::prepare_external_recovery(state, key)
    {
        return adapter_declined();
    }
    if view.text_raster_work().is_some() {
        return match text_atlas::begin(state, view) {
            text_atlas::UiMountedTextWorkOutcome::Ready => {
                state.record_text_pin_frame_observation();
                perform_surface_work(state, view)
            }
            text_atlas::UiMountedTextWorkOutcome::Pending(pending) => {
                text_atlas::settle_deferred(state, view, Some(pending))
            }
            text_atlas::UiMountedTextWorkOutcome::Terminal(outcome) => outcome,
        };
    }
    perform_surface_work(state, view)
}

fn perform_surface_work(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
) -> UiHostSurfacePresentationOutcome {
    match view.presentation_work() {
        UiMountedPresentationWorkView::Initial(_) => perform_initial(state, view),
        UiMountedPresentationWorkView::Delta(_) => perform_delta(state, view),
        UiMountedPresentationWorkView::Reconstruction(_) => perform_reconstruction(state, view),
        UiMountedPresentationWorkView::Sample(_) => sample::perform_sample(state, view),
        UiMountedPresentationWorkView::Unchanged(unchanged) => {
            retained_frame::perform_unchanged(state, view, unchanged)
        }
    }
}

fn perform_reconstruction(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
) -> UiHostSurfacePresentationOutcome {
    let key = view.binding().diagnostic_value();
    let recovery = if state.lifecycle.recovery_required(key) {
        let Some(recovery) = state.lifecycle.take_recovery(key) else {
            return require_owner_reconstruction(state, key);
        };
        Some(recovery)
    } else {
        None
    };
    let defer_initial_observation = defer_presentation_initial_observation(state);
    let Some(device) = state.device.as_ref() else {
        if let Some(recovery) = recovery {
            state.lifecycle.restore_recovery(recovery);
        }
        return adapter_declined();
    };
    let Some(surface) = state.presentation_surface.as_ref() else {
        if let Some(recovery) = recovery {
            state.lifecycle.restore_recovery(recovery);
        }
        return adapter_declined();
    };
    let mut graphics = crate::native::UiNativePresentationAccess::new(device, surface);
    let result = present_cold_reconstruction::<UiWgpuNativePresentationPort>(
        &mut graphics,
        &mut state.resources,
        &mut state.physical_signal,
        &state.text_atlas,
        state.text_atlas_gpu.as_ref(),
        view,
        recovery,
        defer_initial_observation,
        &mut state.lifecycle,
    );
    let (cost, retained, pixels, port_crossings, recovery) = match result {
        Ok(reconstruction) => reconstruction.into_parts(),
        Err(UiNativeReconstructionFailure::BeforeEffects {
            denial,
            recovery,
            successor_cause,
        }) => {
            if let Some(recovery) = recovery {
                state.lifecycle.restore_recovery(recovery);
            }
            if let Some(cause) = successor_cause {
                state.lifecycle.require_recovery(key, cause);
            }
            return settle_presentation_failure(
                state,
                view,
                UiNativePresentationFailure::BeforeEffects(denial),
            );
        }
        Err(UiNativeReconstructionFailure::Pending(pending)) => {
            return settle_presentation_failure(
                state,
                view,
                UiNativePresentationFailure::Pending(pending),
            );
        }
    };
    let effects = crate::native::presentation::UiNativePresentationEffects::new(
        true,
        retained.identity_overlay_active(),
    );
    state.retained_draw_lists.insert(key, retained);
    if let Some(recovery) = recovery {
        let _current_recovery = state.lifecycle.settle_recovery(recovery);
    }
    state.lifecycle.record_presented();
    retained_frame::record_retained_frame(
        state,
        view,
        key,
        UiNativePresentationWorkKind::Reconstruction,
        None,
        pixels,
        cost,
        port_crossings,
    );
    let outcome = completed(state, key, view, cost, true, effects);
    #[cfg(feature = "certification-support")]
    if matches!(&outcome, UiHostSurfacePresentationOutcome::Presented(_)) {
        state.record_qualified_derived_state_reconstruction(key);
    }
    outcome
}

fn perform_initial(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
) -> UiHostSurfacePresentationOutcome {
    let key = view.binding().diagnostic_value();
    if state.retained_draw_lists.contains_key(&key) {
        return malformed();
    }
    let defer_initial_observation = defer_presentation_initial_observation(state);
    let Some(device) = state.device.as_ref() else {
        return adapter_declined();
    };
    let Some(surface) = state.presentation_surface.as_ref() else {
        return adapter_declined();
    };
    let mut graphics = crate::native::UiNativePresentationAccess::new(device, surface);
    let result = present_initial::<UiWgpuNativePresentationPort>(
        &mut graphics,
        &mut state.resources,
        &mut state.physical_signal,
        &state.text_atlas,
        state.text_atlas_gpu.as_ref(),
        view,
        defer_initial_observation,
        &mut state.lifecycle,
    );
    let (observation, cost, retained) = match result {
        Ok(presented) => presented.into_parts(),
        Err(failure) => return settle_presentation_failure(state, view, failure),
    };
    state.lifecycle.record_presented();
    state.record_retained_frame_observation(UiNativeRetainedFrameObservation::observed(
        view.frame().diagnostic_value(),
        UiNativePresentationWorkKind::Initial,
        None,
        [
            observation.retained_baseline_rgba8(),
            observation.retained_center_rgba8(),
        ],
        cost,
        Some(observation.clone()),
    ));
    state.last_presentation = Some(observation);
    let effects = crate::native::presentation::UiNativePresentationEffects::new(
        true,
        retained.identity_overlay_active(),
    );
    state.retained_draw_lists.insert(key, retained);
    completed(state, key, view, cost, true, effects)
}

fn perform_delta(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
) -> UiHostSurfacePresentationOutcome {
    let key = view.binding().diagnostic_value();
    if state.lifecycle.recovery_required(key) {
        return require_owner_reconstruction(state, key);
    }
    let result = present_delta_work(state, view, key);
    let (cost, painted, observed_pixels, port_crossings, effects) = match result {
        Ok(presented) => presented.into_parts(),
        Err(failure) => return settle_presentation_failure(state, view, failure),
    };
    state.lifecycle.resolve_recovery(key);
    state.lifecycle.record_presented();
    let pixels = observed_pixels.unwrap_or_else(|| retained_frame::latest_pixels(state));
    retained_frame::record_retained_frame(
        state,
        view,
        key,
        UiNativePresentationWorkKind::Delta,
        None,
        pixels,
        cost,
        port_crossings,
    );
    completed(state, key, view, cost, painted, effects)
}

fn present_delta_work(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    key: u64,
) -> Result<crate::native::presentation::UiNativeDeltaPresentation, UiNativePresentationFailure> {
    let defer_initial_observation = defer_presentation_initial_observation(state);
    let Some(device) = state.device.as_ref() else {
        return Err(before_effects_declined());
    };
    let Some(surface) = state.presentation_surface.as_ref() else {
        return Err(before_effects_declined());
    };
    let mut graphics = crate::native::UiNativePresentationAccess::new(device, surface);
    let Some(retained) = state.retained_draw_lists.get_mut(&key) else {
        return Err(before_effects_malformed());
    };
    present_delta::<UiWgpuNativePresentationPort>(
        &mut graphics,
        &mut state.resources,
        &mut state.physical_signal,
        &state.text_atlas,
        state.text_atlas_gpu.as_ref(),
        view,
        retained,
        defer_initial_observation,
        &mut state.lifecycle,
    )
}

fn defer_presentation_initial_observation(state: &mut UiNativeHostState) -> bool {
    #[cfg(feature = "certification-support")]
    {
        state
            .qualification
            .defer_next_presentation_initial_observation()
    }
    #[cfg(not(feature = "certification-support"))]
    {
        let _ = state;
        false
    }
}

fn require_owner_reconstruction(
    state: &mut UiNativeHostState,
    key: u64,
) -> UiHostSurfacePresentationOutcome {
    state.retained_draw_lists.remove(&key);
    UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired,
    )
}

fn settle_presentation_failure(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    failure: UiNativePresentationFailure,
) -> UiHostSurfacePresentationOutcome {
    match failure {
        UiNativePresentationFailure::BeforeEffects(denial) => {
            UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial)
        }
        UiNativePresentationFailure::RecoveryRequired { denial, cause } => {
            state.require_surface_reconstruction(cause);
            UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial)
        }
        UiNativePresentationFailure::Pending(mut pending) => {
            state.lifecycle.record_presentation_stage(
                crate::native::UiNativePresentationEffectPhase::PresentHandoff,
            );
            state
                .captures
                .invalidate_source(view.binding().diagnostic_value());
            let token = view.issue_completion_token();
            if !pending.bind_completion_identity(token.diagnostic_value()) {
                return mark_presentation_indeterminate(state);
            }
            let _remembered = state.lifecycle.remember_pending_presentation(
                view.protocol(),
                view.host_session_identity(),
                view.requirement().host_surface(),
                view.binding(),
                token.diagnostic_value(),
            );
            #[cfg(feature = "certification-support")]
            {
                let qualification = state
                    .qualification
                    .presentation_external_qualification(pending.physical_work());
                pending.qualify_external_observation(
                    qualification.effects_indeterminate(),
                    qualification.duplicate_completed(),
                );
            }
            state.pending_presentations.push(pending);
            UiHostSurfacePresentationOutcome::InFlight(token)
        }
    }
}

#[cfg(test)]
#[path = "presentation_tests.rs"]
mod tests;
