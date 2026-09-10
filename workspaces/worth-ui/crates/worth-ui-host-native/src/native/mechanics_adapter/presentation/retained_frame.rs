use worth_ui_host_contract::{
    UiHostPresentationCostReport, UiHostSurfacePresentationOutcome, UiMountedFrameConsumptionView,
    UiMountedPresentationUnchanged,
};

use crate::native::{
    presentation::observation_for_retained, UiNativeHostState, UiNativePresentationWorkKind,
    UiNativeRetainedFrameObservation,
};

pub(super) fn perform_unchanged(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    unchanged: &UiMountedPresentationUnchanged,
) -> UiHostSurfacePresentationOutcome {
    let key = view.binding().diagnostic_value();
    if state.lifecycle.recovery_required(key) {
        return super::require_owner_reconstruction(state, key);
    }
    if view.appearance_work().is_some() {
        return present_appearance_only(state, view, unchanged, key);
    }
    retain_unchanged(state, view, unchanged, key)
}

fn present_appearance_only(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    unchanged: &UiMountedPresentationUnchanged,
    key: u64,
) -> UiHostSurfacePresentationOutcome {
    let defer_initial_observation = super::defer_presentation_initial_observation(state);
    let Some(device) = state.device.as_ref() else {
        return super::adapter_declined();
    };
    let Some(surface) = state.presentation_surface.as_ref() else {
        return super::adapter_declined();
    };
    let mut graphics = crate::native::UiNativePresentationAccess::new(device, surface);
    let Some(retained) = state.retained_draw_lists.get_mut(&key) else {
        return super::malformed();
    };
    let result = crate::native::presentation::present_unchanged_appearance::<
        crate::native::presentation::UiWgpuNativePresentationPort,
    >(
        &mut graphics,
        &mut state.resources,
        &mut state.physical_signal,
        &state.text_atlas,
        state.text_atlas_gpu.as_ref(),
        view,
        unchanged,
        retained,
        defer_initial_observation,
        &mut state.lifecycle,
    );
    let (cost, painted, observed_pixels, port_crossings, effects) = match result {
        Ok(presented) => presented.into_parts(),
        Err(failure) => return super::settle_presentation_failure(state, view, failure),
    };
    state.lifecycle.resolve_recovery(key);
    state.lifecycle.record_presented();
    let pixels = observed_pixels.unwrap_or_else(|| latest_pixels(state));
    record_retained_frame(
        state,
        view,
        key,
        UiNativePresentationWorkKind::Unchanged,
        None,
        pixels,
        cost,
        port_crossings,
    );
    super::completed(state, key, view, cost, painted, effects)
}

fn retain_unchanged(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    unchanged: &UiMountedPresentationUnchanged,
    key: u64,
) -> UiHostSurfacePresentationOutcome {
    let Some(retained) = state.retained_draw_lists.get_mut(&key) else {
        return super::malformed();
    };
    if retained.apply_unchanged(unchanged).is_err() {
        return super::malformed();
    }
    let pixels = latest_pixels(state);
    record_retained_frame(
        state,
        view,
        key,
        UiNativePresentationWorkKind::Unchanged,
        None,
        pixels,
        Default::default(),
        0,
    );
    super::completed(
        state,
        key,
        view,
        Default::default(),
        false,
        crate::native::presentation::UiNativePresentationEffects::default(),
    )
}

pub(super) fn record_retained_frame(
    state: &mut UiNativeHostState,
    view: &UiMountedFrameConsumptionView<'_>,
    key: u64,
    kind: UiNativePresentationWorkKind,
    sample_presentation_epoch: Option<worth_ui_host_contract::UiHostPresentationEpoch>,
    pixels: [[u8; 4]; 2],
    cost: UiHostPresentationCostReport,
    port_crossings: u8,
) {
    let access = state.presentation_access();
    let observation = access
        .as_ref()
        .zip(state.retained_draw_lists.get(&key))
        .and_then(|(graphics, retained)| {
            observation_for_retained(
                view,
                graphics,
                &state.text_atlas,
                retained,
                pixels,
                cost,
                port_crossings,
            )
        });
    state.record_retained_frame_observation(UiNativeRetainedFrameObservation::observed(
        view.frame().diagnostic_value(),
        kind,
        sample_presentation_epoch,
        pixels,
        cost,
        observation.clone(),
    ));
    state.last_presentation = observation;
}

pub(super) fn latest_pixels(state: &UiNativeHostState) -> [[u8; 4]; 2] {
    state
        .retained_frame_observations
        .last()
        .map(|observation| {
            [
                observation.retained_baseline_rgba8(),
                observation.retained_center_rgba8(),
            ]
        })
        .unwrap_or([[0, 0, 0, 0]; 2])
}
