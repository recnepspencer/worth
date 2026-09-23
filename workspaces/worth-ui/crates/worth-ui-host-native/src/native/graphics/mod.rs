pub(crate) mod adapter_selection;
mod backend;
mod device;

#[cfg(all(test, target_os = "windows"))]
pub(crate) use backend::QUALIFIED_DX12_PRESENTATION_SYSTEM;
pub(crate) use backend::{
    prepare_external_recovery, prepare_platform_graphics, prepare_replacement_target,
    UiNativeBackendDeviceGenerationMechanics, UiNativeBackendDeviceMechanics,
    UiNativeBackendRetainedTarget, UiNativeBackendSurfaceHandle, UiNativeBackendSurfaceMechanics,
    UiNativeGraphicsRecovery, UiNativePreparedGraphicsRecovery,
};
#[cfg(test)]
pub(crate) use backend::{qualified_backend, qualified_backends};
pub(crate) use backend::{qualified_surface_format, qualified_target_format};
pub(crate) use device::{
    UiNativeDeviceGeneration, UiNativeDeviceOwners, UiNativeDeviceState, UiNativeOwnedDevice,
};
