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
        [whole_points(width), whole_points(height)]
    }
}

/// A laid-out length as the whole points that hold it: rounded up, so no
/// fraction of the content is cut off, and held to what `u16` counts.
fn whole_points(length: f32) -> u16 {
    crate::whole_number::whole_u16(f64::from(length.ceil()).clamp(0.0, f64::from(u16::MAX)))
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "content_bounds_tests.rs"]
mod tests;
