use crate::native::graphics::UiNativeDeviceState;
use crate::native::presentation::UiNativePresentationSurface;

/// Contractual device/surface preparation boundary. Implementations return
/// external observations and owned mechanics, never a lifecycle verdict.
pub(crate) trait UiNativeGraphicsPort {
    type Window;
    type Device;
    type Surface;
    type Prepared;
    type Recovery;
    type Target;

    fn prepare(window: Self::Window) -> Result<Self::Prepared, UiNativeGraphicsPortDenial>;

    fn replacement_target(
        device: &Self::Device,
        scale_factor: f64,
        extent: [u32; 2],
    ) -> Self::Target;

    fn prepare_external_recovery(
        device: &Self::Device,
        surface: &Self::Surface,
        window: Self::Window,
        recovery: UiNativeGraphicsRecovery,
    ) -> Result<Self::Recovery, UiNativeGraphicsPortDenial>;
}

/// The window graphics prepare for: a platform window, whose swapchain they
/// create, or an offscreen client area of the given extent and scale.
pub(crate) enum UiNativeGraphicsWindow {
    Platform(std::sync::Arc<winit::window::Window>),
    Offscreen { extent: [u32; 2], scale_factor: f64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeGraphicsRecovery {
    SurfaceOutdated,
    SurfaceLost,
    DeviceLost,
}

pub(crate) struct UiNativePreparedGraphics {
    device: UiNativeDeviceState,
    surface: UiNativePresentationSurface,
    crossing_count: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeGraphicsPortDenial {
    Surface,
    Adapter,
    Device,
}

impl UiNativePreparedGraphics {
    pub(super) fn new(
        device: UiNativeDeviceState,
        surface: UiNativePresentationSurface,
        crossing_count: u8,
    ) -> Self {
        Self {
            device,
            surface,
            crossing_count,
        }
    }

    pub(crate) fn into_parts(self) -> (UiNativeDeviceState, UiNativePresentationSurface, u8) {
        (self.device, self.surface, self.crossing_count)
    }
}
