/// The surface axes a qualified profile declares, owned by the graphics backend.
///
/// These are separated from [`super::UiNativeAppearanceProfile`] deliberately:
/// appearance rows are consumed by geometry and sized per command kind, while
/// these are consumed by surface configuration and fail as
/// `UiNativeGraphicsPortDenial::Surface`. Different owner, different failure
/// topology, different lifecycle.
///
/// Every axis is a closed enum, not a string: the graphics backend matches on
/// these exhaustively to build its instance and surface configuration, so a
/// profile cannot declare a value the backend has no arm for. The manifest
/// spelling of each variant is `as_str()`, which the closed-record test
/// compares against the parsed TOML field by field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeSurfaceProfile {
    pub backends: UiNativeSurfaceBackends,
    pub surface_format: UiNativeSurfaceFormat,
    pub target_format: UiNativeSurfaceFormat,
    pub present_mode: UiNativePresentMode,
    pub composite_alpha: UiNativeCompositeAlpha,
    pub cpu_adapter: UiNativeCpuAdapterAdmission,
}

/// Whether a software-rasterizing adapter (`DeviceType::Cpu`) may be
/// selected.
///
/// Manifest field: `cpu_adapter`. Every product profile denies it, the X11
/// hardware profile included: a CPU rasterizer presenting for a product is a
/// silent performance fault, and the X11 product manifest's
/// `typed-denial-before-effects-no-fallback` posture forbids falling back to
/// one. Only the software X11 profile (`worth-ui-linux-x11-vulkan-software-v1`,
/// selected by `worth_ui_adapter = "software"`) admits it, because the X
/// server the certification lane runs under (Xvfb, no DRI3) offers hardware
/// adapters no surface formats at all, so there only a software rasterizer
/// can present — measured: Intel `formats=[]`, llvmpipe presents. Ranking is
/// unchanged: a hardware adapter that presents still wins; admission only
/// stops the software one from being refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeCpuAdapterAdmission {
    Deny,
    Allow,
}

impl UiNativeCpuAdapterAdmission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::Allow => "allow",
        }
    }

    /// Const equality for the compile-time binding of the selection arm to
    /// the profile it indexes; see [`super::UiNativeWindowingSystem::same_as`].
    pub(crate) const fn same_as(self, other: Self) -> bool {
        self as u8 == other as u8
    }
}

/// The graphics API family a qualified profile enumerates adapters from.
///
/// Manifest field: `runtime_backend_selection`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeSurfaceBackends {
    Dx12,
    Vulkan,
}

impl UiNativeSurfaceBackends {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dx12 => "Backends::DX12",
            Self::Vulkan => "Backends::VULKAN",
        }
    }
}

/// A texture format a qualified profile pins for its swapchain or its
/// retained presentation target.
///
/// Manifest fields: `surface_format`, `target_format`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeSurfaceFormat {
    Bgra8UnormSrgb,
    Rgba8UnormSrgb,
}

impl UiNativeSurfaceFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bgra8UnormSrgb => "Bgra8UnormSrgb",
            Self::Rgba8UnormSrgb => "Rgba8UnormSrgb",
        }
    }
}

/// The presentation cadence a qualified profile pins.
///
/// Manifest field: `present_mode`. One variant because one is qualified; a
/// second cadence is a new qualification, not a placeholder here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativePresentMode {
    Fifo,
}

impl UiNativePresentMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fifo => "Fifo",
        }
    }
}

/// The per-platform mechanism that satisfies a [`UiNativeClientBackground`].
///
/// Manifest field: `composite_alpha`. `PreMultiplied` is a composited-surface
/// capability — DirectComposition on Windows, `wl_surface` on Wayland — and
/// `Opaque` is what remains where no composited path exists. A mode no
/// qualified profile declares (Metal's `PostMultiplied`) is added with the
/// profile that qualifies it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeCompositeAlpha {
    PreMultiplied,
    Opaque,
}

impl UiNativeCompositeAlpha {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreMultiplied => "PreMultiplied",
            Self::Opaque => "Opaque",
        }
    }
}

/// The product-level requirement a [`UiNativeSurfaceProfile`] must satisfy.
///
/// This is not a surface mechanism and does not belong in that record: the
/// mechanism that satisfies it differs per platform (`PreMultiplied` on a
/// composited Wayland or DirectComposition surface, `Opaque` where no
/// composited path exists), while the requirement itself does not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeClientBackground {
    Transparent,
    Opaque,
}

impl UiNativeClientBackground {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transparent => "transparent",
            Self::Opaque => "opaque",
        }
    }

    /// Renders the requirement for winit's `with_transparent`, which asks the
    /// windowing system for an alpha-capable window. The mechanism that then
    /// satisfies it is the surface profile's `composite_alpha`.
    pub const fn requests_transparent_window(self) -> bool {
        match self {
            Self::Transparent => true,
            Self::Opaque => false,
        }
    }
}
