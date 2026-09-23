#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeAppearanceProfile {
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
