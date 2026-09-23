use std::sync::Arc;

use winit::window::Window;

mod mechanics;
mod surface_selection;

pub(crate) use mechanics::{
    UiNativePreparedGraphicsRecovery, UiWgpuDeviceGenerationMechanics, UiWgpuDeviceMechanics,
    UiWgpuRetainedTarget, UiWgpuSurfaceHandle, UiWgpuSurfaceMechanics,
};
#[cfg(test)]
pub(crate) use surface_selection::{qualified_backend, qualified_backends};
pub(crate) use surface_selection::{qualified_surface_format, qualified_target_format};

use super::port::{
    UiNativeGraphicsPort, UiNativeGraphicsPortDenial, UiNativeGraphicsRecovery,
    UiNativePreparedGraphics,
};
use crate::native::graphics::{
    adapter_selection, UiNativeDeviceGeneration, UiNativeDeviceState, UiNativeOwnedDevice,
};
use crate::native::presentation::{UiNativeOwnedPresentationSurface, UiNativePresentationSurface};

pub(crate) struct UiWgpuNativeGraphicsPort;

/// The surface axes of the compiled profile, read once and named so the
/// instance, capability validation, swapchain and retained target cannot
/// disagree about which record they serve.
const SURFACE: crate::native_profile::UiNativeSurfaceProfile =
    crate::native_profile::WORTH_UI_NATIVE_SURFACE_PROFILE;

/// DirectComposition is how the Windows profile obtains `PreMultiplied`; it
/// is a DX12 option and applies only where DX12 is the qualified backend.
#[cfg(target_os = "windows")]
pub(crate) const QUALIFIED_DX12_PRESENTATION_SYSTEM: wgpu::Dx12SwapchainKind =
    wgpu::Dx12SwapchainKind::DxgiFromVisual;

pub(crate) fn prepare_platform_graphics(
    window: Arc<Window>,
) -> Result<UiNativePreparedGraphics, UiNativeGraphicsPortDenial> {
    UiWgpuNativeGraphicsPort::prepare(window)
}

pub(crate) fn prepare_replacement_target(
    device: &UiNativeOwnedDevice,
    scale_factor: f64,
    extent: [u32; 2],
) -> UiWgpuRetainedTarget {
    UiWgpuNativeGraphicsPort::replacement_target(device, scale_factor, extent)
}

pub(crate) fn prepare_external_recovery(
    device: &UiNativeOwnedDevice,
    surface: &UiNativeOwnedPresentationSurface,
    window: Arc<Window>,
    recovery: UiNativeGraphicsRecovery,
) -> Result<UiNativePreparedGraphicsRecovery, UiNativeGraphicsPortDenial> {
    UiWgpuNativeGraphicsPort::prepare_external_recovery(device, surface, window, recovery)
}

impl UiNativeGraphicsPort for UiWgpuNativeGraphicsPort {
    type Window = Arc<Window>;
    type Device = UiNativeOwnedDevice;
    type Surface = UiNativeOwnedPresentationSurface;
    type Prepared = UiNativePreparedGraphics;
    type Recovery = UiNativePreparedGraphicsRecovery;
    type Target = UiWgpuRetainedTarget;

    fn prepare(
        window: Arc<Window>,
    ) -> Result<UiNativePreparedGraphics, UiNativeGraphicsPortDenial> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = surface_selection::backends(SURFACE.backends);
        #[cfg(target_os = "windows")]
        {
            descriptor.backend_options.dx12.presentation_system =
                QUALIFIED_DX12_PRESENTATION_SYSTEM;
        }
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(|_| UiNativeGraphicsPortDenial::Surface)?;
        let adapter = select_adapter(&instance, &surface)?;
        let adapter_info = adapter.get_info();
        validate_surface_capabilities(&surface.get_capabilities(&adapter))?;
        let descriptor = wgpu::DeviceDescriptor {
            label: Some(crate::native_profile::ACTIVE_PROFILE.device_label),
            required_features: wgpu::Features::empty(),
            required_limits: qualified_required_limits(&adapter),
            ..Default::default()
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&descriptor))
            .map_err(|_| UiNativeGraphicsPortDenial::Device)?;
        let device_lost = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let device_lost_callback = Arc::clone(&device_lost);
        device.set_device_lost_callback(move |_, _| {
            device_lost_callback.store(true, std::sync::atomic::Ordering::Release);
        });
        let size = window.inner_size();
        let surface_suspended = size.width == 0 || size.height == 0;
        let extent = [size.width.max(1), size.height.max(1)];
        let surface_configuration = surface_configuration(extent);
        if !surface_suspended {
            surface.configure(&device, &surface_configuration);
        }
        let retained_target = (!surface_suspended).then(|| retained_target(&device, extent));
        Ok(UiNativePreparedGraphics::new(
            UiNativeDeviceState {
                mechanics: UiWgpuDeviceMechanics {
                    instance,
                    adapter,
                    adapter_info,
                },
                generation: Arc::new(UiNativeDeviceGeneration::new(
                    1,
                    UiWgpuDeviceGenerationMechanics {
                        device,
                        queue,
                        lost: device_lost,
                    },
                )),
            },
            UiNativePresentationSurface {
                mechanics: UiWgpuSurfaceMechanics {
                    surface,
                    retained_target,
                    configuration: surface_configuration,
                },
                scale_factor: window.scale_factor(),
                extent: [size.width, size.height],
                generation: 1,
                suspended: surface_suspended,
            },
            1,
        ))
    }

    fn replacement_target(
        device: &UiNativeOwnedDevice,
        _scale_factor: f64,
        extent: [u32; 2],
    ) -> UiWgpuRetainedTarget {
        UiWgpuRetainedTarget(retained_target(device.state().device(), extent))
    }

    fn prepare_external_recovery(
        device: &UiNativeOwnedDevice,
        surface: &UiNativeOwnedPresentationSurface,
        window: Arc<Window>,
        recovery: UiNativeGraphicsRecovery,
    ) -> Result<UiNativePreparedGraphicsRecovery, UiNativeGraphicsPortDenial> {
        match recovery {
            UiNativeGraphicsRecovery::SurfaceOutdated => {
                Ok(UiNativePreparedGraphicsRecovery::SurfaceOutdated)
            }
            UiNativeGraphicsRecovery::SurfaceLost => {
                let surface = device
                    .state()
                    .mechanics
                    .instance
                    .create_surface(window)
                    .map_err(|_| UiNativeGraphicsPortDenial::Surface)?;
                validate_surface_capabilities(
                    &surface.get_capabilities(&device.state().mechanics.adapter),
                )?;
                Ok(UiNativePreparedGraphicsRecovery::SurfaceLost(
                    UiWgpuSurfaceHandle(surface),
                ))
            }
            UiNativeGraphicsRecovery::DeviceLost => {
                let descriptor = wgpu::DeviceDescriptor {
                    label: Some(crate::native_profile::ACTIVE_PROFILE.recovered_device_label),
                    required_features: wgpu::Features::empty(),
                    required_limits: qualified_required_limits(&device.state().mechanics.adapter),
                    ..Default::default()
                };
                let (successor_device, queue) = pollster::block_on(
                    device.state().mechanics.adapter.request_device(&descriptor),
                )
                .map_err(|_| UiNativeGraphicsPortDenial::Device)?;
                let device_lost = Arc::new(std::sync::atomic::AtomicBool::new(false));
                let callback = Arc::clone(&device_lost);
                successor_device.set_device_lost_callback(move |_, _| {
                    callback.store(true, std::sync::atomic::Ordering::Release);
                });
                let identity = device
                    .state()
                    .generation_identity()
                    .checked_add(1)
                    .ok_or(UiNativeGraphicsPortDenial::Device)?;
                let generation = Arc::new(UiNativeDeviceGeneration::new(
                    identity,
                    UiWgpuDeviceGenerationMechanics {
                        device: successor_device,
                        queue,
                        lost: device_lost,
                    },
                ));
                let retained_target = UiWgpuRetainedTarget(retained_target(
                    generation.device(),
                    surface.state().extent(),
                ));
                Ok(UiNativePreparedGraphicsRecovery::DeviceLost {
                    generation,
                    retained_target,
                })
            }
        }
    }
}

fn select_adapter(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> Result<wgpu::Adapter, UiNativeGraphicsPortDenial> {
    let candidates = pollster::block_on(
        instance.enumerate_adapters(surface_selection::backends(SURFACE.backends)),
    );
    let observed = candidates
        .into_iter()
        .map(|adapter| {
            let info = adapter.get_info();
            (
                adapter_selection::AdapterCandidate {
                    surface_supported: adapter.is_surface_supported(surface),
                    device_type: info.device_type,
                    limits: adapter.limits(),
                    vendor: info.vendor,
                    device: info.device,
                    name: info.name,
                    driver_info: info.driver_info,
                },
                adapter,
            )
        })
        .collect::<Vec<_>>();
    adapter_selection::select_eligible_adapter(observed, SURFACE.cpu_adapter)
        .map(|(_, adapter)| adapter)
        .ok_or(UiNativeGraphicsPortDenial::Adapter)
}

fn validate_surface_capabilities(
    capabilities: &wgpu::SurfaceCapabilities,
) -> Result<(), UiNativeGraphicsPortDenial> {
    surface_selection::surface_offers_profile(capabilities, SURFACE)
        .then_some(())
        .ok_or(UiNativeGraphicsPortDenial::Surface)
}

fn qualified_required_limits(adapter: &wgpu::Adapter) -> wgpu::Limits {
    wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits())
}

fn surface_configuration(extent: [u32; 2]) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_selection::texture_format(SURFACE.surface_format),
        width: extent[0],
        height: extent[1],
        present_mode: surface_selection::present_mode(SURFACE.present_mode),
        desired_maximum_frame_latency: 2,
        alpha_mode: surface_selection::composite_alpha(SURFACE.composite_alpha),
        view_formats: Vec::new(),
    }
}

fn retained_target(device: &wgpu::Device, extent: [u32; 2]) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("worth-ui-retained-presentation-target"),
        size: wgpu::Extent3d {
            width: extent[0].max(1),
            height: extent[1].max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: surface_selection::texture_format(SURFACE.target_format),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}
