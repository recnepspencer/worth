use super::{
    UiNativeAppearanceProfile, UiNativeClientBackground, UiNativeCompositeAlpha,
    UiNativeCpuAdapterAdmission, UiNativePlatformProfileIdentity, UiNativePresentMode,
    UiNativeQualifiedProfile, UiNativeSurfaceBackends, UiNativeSurfaceFormat,
    UiNativeSurfaceProfile, UiNativeWindowingSystem,
};

pub(super) const PROFILE: UiNativeQualifiedProfile = UiNativeQualifiedProfile {
    identity: UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V2,
    manifest: include_str!("../../profiles/worth-ui-windows-dx12-v2.toml"),
    windowing_system: UiNativeWindowingSystem::Win32,
    device_label: "worth-ui-windows-dx12-v2-device",
    recovered_device_label: "worth-ui-windows-dx12-v2-recovered-device",
    client_background: UiNativeClientBackground::Transparent,
    surface: UiNativeSurfaceProfile {
        backends: UiNativeSurfaceBackends::Dx12,
        surface_format: UiNativeSurfaceFormat::Bgra8UnormSrgb,
        target_format: UiNativeSurfaceFormat::Rgba8UnormSrgb,
        present_mode: UiNativePresentMode::Fifo,
        cpu_adapter: UiNativeCpuAdapterAdmission::Deny,
        composite_alpha: UiNativeCompositeAlpha::PreMultiplied,
    },
    appearance: UiNativeAppearanceProfile {
        identity: "worth-ui-windows-dx12-v2",
        version: 2,
        scales_milli: &[1_000, 1_250, 1_500, 2_000],
        anti_alias_fringe_physical_pixels: 1,
        geometry_basis:
            worth_ui_host_contract::UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
        retained_commands: 4_096,
        surface_commands: 2_048,
        outline_commands: 1_024,
        backdrop_commands: 512,
        overlay_order_commands: 4_096,
        pointer_affordance_commands: 64,
        text_foreground_commands: 2_048,
        damage_regions: 4_096,
        primary_pointer: Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
    },
};
