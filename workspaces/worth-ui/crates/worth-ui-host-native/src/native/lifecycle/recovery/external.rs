use super::{UiNativePhysicalRecoveryPreparation, UiNativeRecoveryCause};

pub(crate) fn prepare_external_recovery(
    state: &mut crate::native::UiNativeHostState,
    binding: u64,
) -> bool {
    let Some(preparation) = state.lifecycle.physical_recovery_preparation(binding) else {
        return state.lifecycle.recovery_ready(binding);
    };
    if !prepare_graphics(state, preparation) {
        return false;
    }
    let Some(crate::native::UiNativePresentationOwners { device, surface }) =
        state.presentation_owners.as_ref()
    else {
        return false;
    };
    state.lifecycle.commit_physical_recovery(
        preparation,
        device.state().generation_identity(),
        surface.state().generation(),
    )
}

fn prepare_graphics(
    state: &mut crate::native::UiNativeHostState,
    preparation: UiNativePhysicalRecoveryPreparation,
) -> bool {
    let recovery = match preparation.cause() {
        UiNativeRecoveryCause::SurfaceOutdated => {
            crate::native::UiNativeGraphicsRecovery::SurfaceOutdated
        }
        UiNativeRecoveryCause::SurfaceLost => crate::native::UiNativeGraphicsRecovery::SurfaceLost,
        UiNativeRecoveryCause::DeviceLost => {
            if state.text_atlas_in_flight.is_some()
                || state.text_atlas_recovery.is_some()
                || !state.pending_text_presentations.is_empty()
            {
                return false;
            }
            if let Some(gpu) = state.text_atlas_gpu.take() {
                if let Err(gpu) = gpu.try_close(&mut state.resources) {
                    state.text_atlas_gpu = Some(gpu);
                    return false;
                }
            }
            if !state.text_atlas.clear() {
                return false;
            }
            state.text_pins_by_binding.clear();
            crate::native::UiNativeGraphicsRecovery::DeviceLost
        }
        #[cfg(any(test, feature = "certification-support"))]
        UiNativeRecoveryCause::DerivedStateLost => return false,
        UiNativeRecoveryCause::PresentationIndeterminate
        | UiNativeRecoveryCause::Resize
        | UiNativeRecoveryCause::Dpi => return false,
    };
    let Some(window) = state
        .window
        .as_ref()
        .map(|window| std::sync::Arc::clone(window))
    else {
        return false;
    };
    let Some(crate::native::UiNativePresentationOwners { device, surface }) =
        state.presentation_owners.as_mut()
    else {
        return false;
    };
    if recovery == crate::native::UiNativeGraphicsRecovery::DeviceLost
        && (surface.state().extent().contains(&0) || surface.state().suspended())
    {
        return false;
    }
    let Ok(prepared) =
        crate::native::graphics::prepare_external_recovery(device, surface, window, recovery)
    else {
        return false;
    };
    super::super::surface_succession::commit_external_recovery(
        device,
        surface,
        prepared,
        &mut state.resources,
    )
    .is_ok()
}
