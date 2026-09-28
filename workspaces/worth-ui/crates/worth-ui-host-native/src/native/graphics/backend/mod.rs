mod port;
mod wgpu;

pub(crate) use port::{UiNativeGraphicsRecovery, UiNativeGraphicsWindow};
#[cfg(all(test, target_os = "windows"))]
pub(crate) use wgpu::QUALIFIED_DX12_PRESENTATION_SYSTEM;
pub(crate) use wgpu::{
    prepare_external_recovery, prepare_graphics, prepare_replacement_target,
    UiNativePreparedGraphicsRecovery, UiWgpuAcquiredTexture as UiNativeBackendAcquiredTexture,
    UiWgpuDeviceGenerationMechanics as UiNativeBackendDeviceGenerationMechanics,
    UiWgpuDeviceMechanics as UiNativeBackendDeviceMechanics,
    UiWgpuPresentationTarget as UiNativeBackendPresentationTarget,
    UiWgpuRetainedTarget as UiNativeBackendRetainedTarget,
    UiWgpuSurfaceHandle as UiNativeBackendSurfaceHandle,
    UiWgpuSurfaceMechanics as UiNativeBackendSurfaceMechanics,
};
#[cfg(test)]
pub(crate) use wgpu::{qualified_backend, qualified_backends};
pub(crate) use wgpu::{qualified_surface_format, qualified_target_format};
