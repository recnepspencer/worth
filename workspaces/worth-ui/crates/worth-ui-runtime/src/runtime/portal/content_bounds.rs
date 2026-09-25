use crate::mounting::presentation::UiPublishedRect;

/// Measured occurrence-local content and its finite paint support. Shadows
/// participate in paint coverage without becoming anchor spacing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalContentBounds {
    pub(crate) layout: UiPublishedRect,
    pub(crate) paint: UiPublishedRect,
}

impl UiPortalContentBounds {
    /// The width and height the content lays out over, which a Portal that
    /// fits its content takes as its preferred extent.
    pub(crate) fn layout_extent(self) -> [u16; 2] {
        let [_, _, width, height] = self.layout.components();
        [width as u16, height as u16]
    }
}
