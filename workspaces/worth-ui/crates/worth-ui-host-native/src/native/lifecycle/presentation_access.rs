use crate::native::graphics::{UiNativeDeviceGeneration, UiNativeOwnedDevice};
use crate::native::presentation::UiNativeOwnedPresentationSurface;
use std::sync::Arc;

/// A scoped borrow used while presentation mechanics need both peer owners.
/// It owns neither domain and cannot outlive the lifecycle operation that
/// assembled it.
pub(crate) struct UiNativePresentationAccess<'owners> {
    device: &'owners UiNativeOwnedDevice,
    surface: &'owners UiNativeOwnedPresentationSurface,
}

impl<'owners> UiNativePresentationAccess<'owners> {
    pub(crate) fn new(
        device: &'owners UiNativeOwnedDevice,
        surface: &'owners UiNativeOwnedPresentationSurface,
    ) -> Self {
        Self { device, surface }
    }

    pub(crate) fn device(&self) -> &wgpu::Device {
        self.device.state().device()
    }

    pub(crate) fn queue(&self) -> &wgpu::Queue {
        self.device.state().queue()
    }

    pub(crate) fn device_generation(&self) -> Arc<UiNativeDeviceGeneration> {
        self.device.state().generation()
    }

    pub(crate) fn retained_target(&self) -> &wgpu::Texture {
        self.surface.state().retained_target()
    }

    pub(crate) const fn extent(&self) -> [u32; 2] {
        self.surface.state().extent()
    }

    pub(crate) const fn target_generation(&self) -> u64 {
        self.surface.target_generation()
    }

    pub(crate) fn device_generation_identity(&self) -> u64 {
        self.device.state().generation_identity()
    }

    pub(crate) fn device_lost(&self) -> bool {
        self.device.state().lost()
    }

    pub(crate) const fn surface_generation(&self) -> u64 {
        self.surface.state().generation()
    }

    pub(crate) const fn surface_suspended(&self) -> bool {
        self.surface.state().suspended()
    }

    pub(crate) const fn scale_factor(&self) -> f64 {
        self.surface.state().scale_factor()
    }

    pub(crate) const fn surface_suspensions(&self) -> u64 {
        self.surface.surface_suspensions()
    }

    pub(crate) const fn targetless_surface_suspensions(&self) -> u64 {
        self.surface.targetless_surface_suspensions()
    }

    pub(crate) fn presentation_target(
        &self,
    ) -> &crate::native::graphics::UiNativeBackendPresentationTarget {
        self.surface.state().presentation_target()
    }

    pub(crate) fn surface_configuration(&self) -> &wgpu::SurfaceConfiguration {
        self.surface.state().configuration()
    }

    /// Resizes the swapchain to the committed basis if a basis change is
    /// still pending, before a frame acquires its texture.
    pub(crate) fn configure_pending_surface(&self) {
        self.surface.state().configure_pending(self.device());
    }

    pub(crate) fn adapter_info(&self) -> &wgpu::AdapterInfo {
        self.device.state().adapter_info()
    }

    pub(crate) fn adapter_limits(&self) -> wgpu::Limits {
        self.device.state().adapter_limits()
    }
}
