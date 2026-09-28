use crate::native::graphics::{UiNativeOwnedDevice, UiNativePreparedGraphicsRecovery};
use crate::native::presentation::{
    self, UiNativeOwnedPresentationSurface, UiNativeSurfaceBasisDisposition,
};
use crate::native::{UiNativeResourceClass, UiNativeResourceRegistry};

pub(crate) fn resize_surface(
    device: &UiNativeOwnedDevice,
    surface: &mut UiNativeOwnedPresentationSurface,
    extent: [u32; 2],
    registry: &mut UiNativeResourceRegistry,
) -> Result<bool, ()> {
    let _stage =
        crate::native::trace_resize_stage(crate::native::UiNativeResizeTraceStage::Surface);
    replace_basis(
        device,
        surface,
        surface.state().scale_factor(),
        extent,
        registry,
    )
}

pub(crate) fn rebind_surface_scale(
    device: &UiNativeOwnedDevice,
    surface: &mut UiNativeOwnedPresentationSurface,
    scale_factor: f64,
    extent: [u32; 2],
    registry: &mut UiNativeResourceRegistry,
) -> Result<bool, ()> {
    replace_basis(device, surface, scale_factor, extent, registry)
}

#[cfg(feature = "certification-support")]
pub(crate) fn replace_retained_target_for_reconstruction(
    device: &UiNativeOwnedDevice,
    surface: &mut UiNativeOwnedPresentationSurface,
    registry: &mut UiNativeResourceRegistry,
) -> Result<(), ()> {
    let extent = surface.state().extent();
    if extent.contains(&0) || surface.state().suspended() {
        return Err(());
    }
    replace_target(
        device,
        surface,
        surface.state().scale_factor(),
        extent,
        registry,
    )
}

pub(crate) fn collect_settled_device_generations(
    device: &mut UiNativeOwnedDevice,
    registry: &mut UiNativeResourceRegistry,
) -> Result<(), ()> {
    device.collect_settled(registry)
}

pub(crate) fn commit_external_recovery(
    device: &mut UiNativeOwnedDevice,
    surface: &mut UiNativeOwnedPresentationSurface,
    prepared: UiNativePreparedGraphicsRecovery,
    registry: &mut UiNativeResourceRegistry,
) -> Result<(), ()> {
    commit_prepared_recovery(device, surface, prepared, registry)
}

fn replace_basis(
    device: &UiNativeOwnedDevice,
    surface: &mut UiNativeOwnedPresentationSurface,
    scale_factor: f64,
    extent: [u32; 2],
    registry: &mut UiNativeResourceRegistry,
) -> Result<bool, ()> {
    match presentation::surface_basis::classify(
        surface.state().scale_factor(),
        surface.state().extent(),
        scale_factor,
        extent,
    ) {
        UiNativeSurfaceBasisDisposition::Unchanged => Ok(false),
        UiNativeSurfaceBasisDisposition::Suspend => {
            surface.suspend(scale_factor, extent, registry)?;
            Ok(true)
        }
        UiNativeSurfaceBasisDisposition::Replace => {
            replace_target(device, surface, scale_factor, extent, registry)?;
            Ok(true)
        }
    }
}

fn replace_target(
    device: &UiNativeOwnedDevice,
    surface: &mut UiNativeOwnedPresentationSurface,
    scale_factor: f64,
    extent: [u32; 2],
    registry: &mut UiNativeResourceRegistry,
) -> Result<(), ()> {
    let successor_owner = registry.register(UiNativeResourceClass::RetainedTarget)?;
    let successor =
        crate::native::graphics::prepare_replacement_target(device, scale_factor, extent);
    surface.replace_basis(successor, successor_owner, scale_factor, extent, registry)
}

fn commit_prepared_recovery(
    device: &mut UiNativeOwnedDevice,
    surface: &mut UiNativeOwnedPresentationSurface,
    prepared: UiNativePreparedGraphicsRecovery,
    registry: &mut UiNativeResourceRegistry,
) -> Result<(), ()> {
    match prepared {
        UiNativePreparedGraphicsRecovery::SurfaceOutdated => {
            surface.state().configure(device.state().device());
        }
        UiNativePreparedGraphicsRecovery::SurfaceLost(successor) => {
            let successor_owner = registry.register(UiNativeResourceClass::Surface)?;
            surface.replace_surface(successor, successor_owner, registry)?;
            surface.state().configure(device.state().device());
        }
        UiNativePreparedGraphicsRecovery::DeviceLost {
            generation,
            retained_target,
        } => {
            if surface.state().extent().contains(&0) || surface.state().suspended() {
                return Err(());
            }
            let mut owners = registry.reserve(&[
                UiNativeResourceClass::Device,
                UiNativeResourceClass::Queue,
                UiNativeResourceClass::RetainedTarget,
            ])?;
            let retained_target_owner = owners.pop().expect("retained-target owner");
            let queue_owner = owners.pop().expect("queue owner");
            let device_owner = owners.pop().expect("device owner");
            surface.state().configure(generation.device());
            device.replace_generation(generation, device_owner, queue_owner);
            surface.replace_target(retained_target, retained_target_owner, registry)?;
        }
    }
    Ok(())
}
