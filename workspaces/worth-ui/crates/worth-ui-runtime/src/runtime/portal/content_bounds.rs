use worth_ui_host_contract::UiMountedCanonicalBox;

/// Measured occurrence-local content and its finite paint support. Shadows
/// participate in paint coverage without becoming anchor spacing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPortalContentBounds {
    pub(crate) layout: UiMountedCanonicalBox,
    pub(crate) paint: UiMountedCanonicalBox,
}

impl Eq for UiPortalContentBounds {}
