#![allow(
    dead_code,
    reason = "Gate 1 retains the native appearance profile for later host qualification"
)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativePlatformProfileIdentity(&'static str);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeMechanicsCapacities {
    pub retained_commands: u16,
    pub rectangle_commands: u16,
    pub text_commands: u16,
    pub damage_regions: u16,
    pub order_edits: u16,
    pub text_bytes: u32,
    pub readiness_owners: u8,
    pub resource_registry_entries: u8,
    pub causes_per_owner: u8,
    pub ready_owner_slots: u8,
    pub presentation_slots: u8,
    pub readback_slots: u8,
    pub readback_bytes: u32,
}

pub const WORTH_UI_NATIVE_PROFILE_MANIFEST: &str =
    include_str!("../profiles/worth-ui-windows-dx12-v1.toml");
pub(crate) const WORTH_UI_NATIVE_NEXT_PROFILE_MANIFEST: &str =
    include_str!("../profiles/worth-ui-windows-dx12-v2.toml");
pub(crate) const QUALIFIED_WHEEL_LINE_LOGICAL_SUBPIXELS: i64 = 40_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeStagedAppearanceProfile {
    pub(crate) identity: &'static str,
    pub(crate) version: u16,
    pub(crate) scales_milli: &'static [u16; 4],
    pub(crate) anti_alias_fringe_physical_pixels: u8,
    pub(crate) geometry_basis: worth_ui_host_contract::UiHostAppearanceGeometryQualificationBasis,
    pub(crate) retained_commands: u16,
    pub(crate) surface_commands: u16,
    pub(crate) outline_commands: u16,
    pub(crate) backdrop_commands: u16,
    pub(crate) overlay_order_commands: u16,
    pub(crate) pointer_affordance_commands: u16,
    pub(crate) text_foreground_commands: u16,
    pub(crate) damage_regions: u16,
    pub(crate) primary_pointer: Option<worth_ui_host_contract::UiHostPrimaryPointerKind>,
}

pub(crate) const STAGED_APPEARANCE_PROFILE: UiNativeStagedAppearanceProfile =
    UiNativeStagedAppearanceProfile {
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
    };

impl UiNativePlatformProfileIdentity {
    pub const WORTH_UI_WINDOWS_DX12_V1: Self = Self("worth-ui-windows-dx12-v1");

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl UiNativeMechanicsCapacities {
    pub const QUALIFIED: Self = Self {
        retained_commands: 4_096,
        rectangle_commands: 2_048,
        text_commands: 2_048,
        damage_regions: 4_096,
        order_edits: 4_096,
        text_bytes: 1_048_576,
        readiness_owners: 8,
        resource_registry_entries: 32,
        causes_per_owner: 64,
        ready_owner_slots: 8,
        presentation_slots: 2,
        readback_slots: 4,
        readback_bytes: 16_777_216,
    };
}
