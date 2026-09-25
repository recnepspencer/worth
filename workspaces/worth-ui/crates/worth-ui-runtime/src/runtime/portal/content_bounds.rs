use crate::mounting::presentation::UiPublishedRect;

/// Measured occurrence-local content and its finite paint support. Shadows
/// participate in paint coverage without becoming anchor spacing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalContentBounds {
    pub(crate) layout: UiPublishedRect,
    pub(crate) paint: UiPublishedRect,
}
