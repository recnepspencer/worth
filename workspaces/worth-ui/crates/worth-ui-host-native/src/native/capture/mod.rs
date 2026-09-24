mod admission;
mod cancellation;
#[cfg(feature = "certification-support")]
mod certification;
mod completion;
mod port;
mod readback;
mod recovery;
mod source;
mod state;
#[cfg(test)]
mod tests;

#[cfg(feature = "certification-support")]
pub use certification::{UiNativeCaptureExternalObservation, UiNativeCaptureProtocolWorld};
pub(crate) use state::UiNativeCaptureState;

pub(crate) fn capability() -> worth_ui_host_contract::UiHostCaptureCapability {
    worth_ui_host_contract::UiHostCaptureCapability::Pixels {
        maximum_bytes: u64::from(crate::UiNativeMechanicsCapacities::QUALIFIED.readback_bytes),
        exact_presentation_epoch: true,
    }
}

pub(crate) fn observe(
    state: &mut crate::native::UiNativeHostState,
    request: worth_ui_host_contract::UiHostVisualCaptureRequest,
) -> worth_ui_host_contract::UiHostCaptureObservationOutcome {
    let outcome = {
        let crate::native::UiNativeHostState {
            presentation_owners,
            captures,
            resources,
            ..
        } = state;
        let access = presentation_owners
            .as_ref()
            .map(crate::native::UiNativePresentationOwners::access);
        captures.observe(access.as_ref(), resources, request)
    };
    collect_settled_graphics_generations(state);
    outcome
}

pub(crate) fn cancel(
    state: &mut crate::native::UiNativeHostState,
    request: worth_ui_host_contract::UiHostVisualCaptureRequest,
) -> worth_ui_host_contract::UiHostCaptureCancellationOutcome {
    let outcome = {
        let crate::native::UiNativeHostState {
            presentation_owners,
            captures,
            resources,
            ..
        } = state;
        let access = presentation_owners
            .as_ref()
            .map(crate::native::UiNativePresentationOwners::access);
        captures.cancel(access.as_ref(), resources, request)
    };
    collect_settled_graphics_generations(state);
    outcome
}

pub(crate) fn close(state: &mut crate::native::UiNativeHostState) -> bool {
    let crate::native::UiNativeHostState {
        captures,
        presentation_owners,
        resources,
        ..
    } = state;
    let access = presentation_owners
        .as_ref()
        .map(crate::native::UiNativePresentationOwners::access);
    captures.close(access.as_ref(), resources);
    let settled = captures.is_settled();
    if settled {
        if let Some(crate::native::UiNativePresentationOwners { device, .. }) =
            presentation_owners.as_mut()
        {
            let _ = crate::native::lifecycle::collect_settled_device_generations(device, resources);
        }
    }
    settled
}

fn collect_settled_graphics_generations(state: &mut crate::native::UiNativeHostState) {
    if let Some(crate::native::UiNativePresentationOwners { device, .. }) =
        state.presentation_owners.as_mut()
    {
        let _ = crate::native::lifecycle::collect_settled_device_generations(
            device,
            &mut state.resources,
        );
    }
}

pub(crate) fn record_completed_view(
    state: &mut crate::native::UiNativeHostState,
    view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    epoch: worth_ui_host_contract::UiHostPresentationEpoch,
) {
    record_completed(
        state,
        view.host_session_identity(),
        view.frame(),
        view.attempt(),
        view.requirement().host_surface(),
        view.binding(),
        epoch,
    );
}

pub(crate) fn record_completed_basis(
    state: &mut crate::native::UiNativeHostState,
    basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    epoch: worth_ui_host_contract::UiHostPresentationEpoch,
) {
    record_completed(
        state,
        basis.host_session_identity(),
        frame,
        basis.attempt(),
        basis.host_surface(),
        basis.binding(),
        epoch,
    );
}

#[allow(clippy::too_many_arguments)]
fn record_completed(
    state: &mut crate::native::UiNativeHostState,
    host_session: u64,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    epoch: worth_ui_host_contract::UiHostPresentationEpoch,
) {
    let key = binding.diagnostic_value();
    let Some(regions) = state
        .retained_draw_lists
        .get(&key)
        .and_then(crate::native::UiNativeRetainedDrawList::realized_regions)
    else {
        state.captures.invalidate_source(binding.diagnostic_value());
        return;
    };
    let access = state.presentation_access();
    let transform = state
        .window
        .as_ref()
        .zip(access.as_ref())
        .and_then(|(window, access)| source::coordinate_transform(window, access));
    state.captures.record_source(
        binding,
        source::UiNativeCaptureSource::completed(source::UiNativeCaptureSourceInput {
            host_session,
            frame,
            attempt,
            host_surface,
            binding,
            epoch,
            transform,
            regions,
        }),
    );
}
