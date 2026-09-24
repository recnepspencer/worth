use worth_ui_host_contract::{UiHostSurfacePresentationOutcome, UiMountedFrameConsumptionView};

use crate::native::{
    presentation::{
        present_sample, UiNativePresentationFailure, UiNativeSamplePresentation,
        UiWgpuNativePresentationPort,
    },
    UiNativeHostState, UiNativePresentationWorkKind,
};

use super::{
    before_effects_declined, before_effects_malformed, completed,
    defer_presentation_initial_observation, require_owner_reconstruction, retained_frame,
    settle_presentation_failure,
};

pub(super) fn perform_sample(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
) -> UiHostSurfacePresentationOutcome {
    let key = view.binding().diagnostic_value();
    if state.lifecycle.recovery_required(key) {
        return require_owner_reconstruction(state, key);
    }
    let result = present_sample_work(state, view, key);
    let (cost, paint, port_crossings, effects) = match result {
        Ok(presented) => presented.into_parts(),
        Err(failure) => return settle_presentation_failure(state, view, failure),
    };
    state.lifecycle.resolve_recovery(key);
    state.lifecycle.record_presented();
    let painted = paint.painted();
    let pixels = paint
        .pixels()
        .unwrap_or_else(|| retained_frame::latest_pixels(state));
    let outcome = completed(state, key, view, cost, painted, effects);
    let sample_presentation_epoch = match &outcome {
        UiHostSurfacePresentationOutcome::Presented(completion) => Some(completion.epoch()),
        _ => None,
    };
    retained_frame::record_retained_frame(
        state,
        view,
        key,
        UiNativePresentationWorkKind::Sample,
        sample_presentation_epoch,
        pixels,
        cost,
        port_crossings,
    );
    outcome
}

fn present_sample_work(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    key: u64,
) -> Result<UiNativeSamplePresentation, UiNativePresentationFailure> {
    let defer_initial_observation = defer_presentation_initial_observation(state);
    let Some(crate::native::UiNativePresentationOwners { device, surface }) =
        state.presentation_owners.as_ref()
    else {
        return Err(before_effects_declined());
    };
    let mut graphics = crate::native::UiNativePresentationAccess::new(device, surface);
    let Some(retained) = state.retained_draw_lists.get_mut(&key) else {
        return Err(before_effects_malformed());
    };
    present_sample::<UiWgpuNativePresentationPort>(
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
