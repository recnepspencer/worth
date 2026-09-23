//! Maps the qualified surface profile's closed axes onto wgpu's vocabulary.
//!
//! This is the only place the profile's surface enums meet wgpu types. The
//! profile module owns the qualified record and may not name a vendor; this
//! module owns the vendor and matches each axis exhaustively, so a profile
//! variant with no wgpu counterpart fails to compile here rather than being
//! configured as some default.

use crate::native_profile::{
    UiNativeCompositeAlpha, UiNativePresentMode, UiNativeSurfaceBackends, UiNativeSurfaceFormat,
    UiNativeSurfaceProfile,
};

pub(crate) fn backends(selection: UiNativeSurfaceBackends) -> wgpu::Backends {
    match selection {
        UiNativeSurfaceBackends::Dx12 => wgpu::Backends::DX12,
        UiNativeSurfaceBackends::Vulkan => wgpu::Backends::VULKAN,
    }
}

/// The single backend an adapter enumerated under [`backends`] must report.
#[cfg(test)]
fn backend(selection: UiNativeSurfaceBackends) -> wgpu::Backend {
    match selection {
        UiNativeSurfaceBackends::Dx12 => wgpu::Backend::Dx12,
        UiNativeSurfaceBackends::Vulkan => wgpu::Backend::Vulkan,
    }
}

pub(crate) fn texture_format(format: UiNativeSurfaceFormat) -> wgpu::TextureFormat {
    match format {
        UiNativeSurfaceFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        UiNativeSurfaceFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
    }
}

pub(crate) fn present_mode(mode: UiNativePresentMode) -> wgpu::PresentMode {
    match mode {
        UiNativePresentMode::Fifo => wgpu::PresentMode::Fifo,
    }
}

pub(crate) fn composite_alpha(alpha: UiNativeCompositeAlpha) -> wgpu::CompositeAlphaMode {
    match alpha {
        UiNativeCompositeAlpha::PreMultiplied => wgpu::CompositeAlphaMode::PreMultiplied,
        UiNativeCompositeAlpha::Opaque => wgpu::CompositeAlphaMode::Opaque,
    }
}

/// Whether a surface offers every mode the qualified profile requires.
///
/// This is a capability check, not a windowing check: a Wayland surface also
/// offers `Opaque`, so an X11 profile landing on a Wayland session passes here.
/// Refusing that mismatch is the windowing enforcement's job at event-loop
/// construction, where the fact is knowable.
pub(crate) fn surface_offers_profile(
    capabilities: &wgpu::SurfaceCapabilities,
    profile: UiNativeSurfaceProfile,
) -> bool {
    capabilities
        .formats
        .contains(&texture_format(profile.surface_format))
        && capabilities
            .present_modes
            .contains(&present_mode(profile.present_mode))
        && capabilities
            .alpha_modes
            .contains(&composite_alpha(profile.composite_alpha))
}

/// The backends the compiled profile enumerates, for test devices that must
/// open the same backend the product does.
#[cfg(test)]
pub(crate) fn qualified_backends() -> wgpu::Backends {
    backends(crate::native_profile::WORTH_UI_NATIVE_SURFACE_PROFILE.backends)
}

/// The single backend an adapter opened for the compiled profile must report.
#[cfg(test)]
pub(crate) fn qualified_backend() -> wgpu::Backend {
    backend(crate::native_profile::WORTH_UI_NATIVE_SURFACE_PROFILE.backends)
}

/// The format of the retained target every presentation pipeline renders into.
///
/// Pipelines declare their color target at creation and wgpu refuses a render
/// pass whose attachment disagrees, so this is read from the same record that
/// creates the target rather than restated beside each pipeline.
pub(crate) fn qualified_target_format() -> wgpu::TextureFormat {
    texture_format(crate::native_profile::WORTH_UI_NATIVE_SURFACE_PROFILE.target_format)
}

/// The swapchain format the retained-to-surface transfer pipeline targets.
pub(crate) fn qualified_surface_format() -> wgpu::TextureFormat {
    texture_format(crate::native_profile::WORTH_UI_NATIVE_SURFACE_PROFILE.surface_format)
}
