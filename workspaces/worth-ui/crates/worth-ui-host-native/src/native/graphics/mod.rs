pub(crate) mod adapter_selection;
mod backend;
mod device;

#[cfg(all(test, target_os = "windows"))]
pub(crate) use backend::QUALIFIED_DX12_PRESENTATION_SYSTEM;
pub(crate) use backend::{
    prepare_external_recovery, prepare_graphics, prepare_replacement_target,
    UiNativeBackendAcquiredTexture, UiNativeBackendDeviceGenerationMechanics,
    UiNativeBackendDeviceMechanics, UiNativeBackendPresentationTarget,
    UiNativeBackendRetainedTarget, UiNativeBackendSurfaceHandle, UiNativeBackendSurfaceMechanics,
    UiNativeGraphicsRecovery, UiNativeGraphicsWindow, UiNativePreparedGraphicsRecovery,
};
#[cfg(test)]
pub(crate) use backend::{qualified_backend, qualified_backends};
pub(crate) use backend::{qualified_surface_format, qualified_target_format};
pub(crate) use device::{
    UiNativeDeviceGeneration, UiNativeDeviceOwners, UiNativeDeviceState, UiNativeOwnedDevice,
};
