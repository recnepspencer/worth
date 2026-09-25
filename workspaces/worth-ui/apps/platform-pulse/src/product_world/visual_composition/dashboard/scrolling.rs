//! Authored content extents and region viewports for the two independent
//! dashboard lists.
//!
//! A panel's viewport is the placement its owner declares for its Mosaic
//! region. It is never reconstructed from the content it clips, so content
//! that overflows is overflow the region can actually prove.
use worth_ui::facade::declaration::{ComponentViewportAxisPlacement, ComponentViewportRegion};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DashboardScrollPanel {
    ServiceHealth,
    RecentActivity,
}

impl DashboardScrollPanel {
    pub const ALL: [Self; 2] = [Self::ServiceHealth, Self::RecentActivity];
    pub const fn owner(self) -> &'static str {
        match self {
            Self::ServiceHealth => "health_content",
            Self::RecentActivity => "activity_content",
        }
    }
    /// The authored extent of the panel's scrolled content, in host-surface
    /// logical points. Service health stays a vertical list: its inline extent
    /// equals its region's, so it keeps block-only travel. Recent activity
    /// carries twelve rows and four trailing columns, so it overflows on both.
    pub const fn content_rect(self) -> [u16; 4] {
        match self {
            Self::ServiceHealth => [1120, 324, 360, 344],
            Self::RecentActivity => [290, 693, 1560, 672],
        }
    }
    /// The declared line extent a keyboard step or one wheel line moves this
    /// panel, in logical points. It is a declared product decision, never an
    /// incidental row height.
    pub const fn line_extent_logical_points(self) -> u16 {
        match self {
            Self::ServiceHealth => 20,
            Self::RecentActivity => 20,
        }
    }
    pub const fn region(self) -> &'static str {
        match self {
            Self::ServiceHealth => "platform.pulse.mosaic.region.service_list",
            Self::RecentActivity => "platform.pulse.mosaic.region.activity_list",
        }
    }
    /// The panel's region viewport, in host-surface logical points. Service
    /// health keeps its region as wide as its content, so it stays a vertical
    /// list; Recent activity's region is narrower than its content, so the
    /// list travels on both axes.
    pub const fn region_rect(self) -> [u16; 4] {
        match self {
            Self::ServiceHealth => [1_120, 324, 360, 269],
            Self::RecentActivity => [290, 693, 768, 269],
        }
    }
    /// The placement the panel owner declares for its region. The owner is
    /// placed against the viewport, so the region is too.
    pub fn region_placement(self) -> ComponentViewportRegion {
        let [x, y, width, height] = self.region_rect();
        ComponentViewportRegion::new(
            ComponentViewportAxisPlacement::fixed_from_start(x, width)
                .expect("Pulse scroll region width is nonzero"),
            ComponentViewportAxisPlacement::fixed_from_start(y, height)
                .expect("Pulse scroll region height is nonzero"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::dashboard_elements;
    use super::DashboardScrollPanel;

    /// The horizontal slack the authored content may leave beyond its last
    /// element before the overflow stops being real content.
    const TRAILING_SLACK_LOGICAL_POINTS: u32 = 32;

    fn extent(panel: DashboardScrollPanel) -> (u32, u32, u32, u32) {
        let [_, _, content_width, content_height] = panel.content_rect();
        let [_, _, region_width, region_height] = panel.region_rect();
        (
            u32::from(content_width),
            u32::from(content_height),
            u32::from(region_width),
            u32::from(region_height),
        )
    }

    #[test]
    fn recent_activity_content_exceeds_its_region_viewport_on_both_axes() {
        let (content_width, content_height, region_width, region_height) =
            extent(DashboardScrollPanel::RecentActivity);
        assert!(
            content_height >= 2 * region_height,
            "content height {content_height} is under two viewports of {region_height}",
        );
        assert!(
            content_width >= 2 * region_width,
            "content width {content_width} is under two viewports of {region_width}",
        );
    }

    /// The scrolled children are authored in content-local coordinates, so the
    /// rightmost authored edge is measured against the content extent itself,
    /// not against a host-surface position the product no longer states.
    #[test]
    fn recent_activity_authors_content_out_to_its_inline_extent() {
        let panel = DashboardScrollPanel::RecentActivity;
        let [_, _, content_width, _] = panel.content_rect();
        let content_width = u32::from(content_width);
        let rightmost = dashboard_elements()
            .into_iter()
            .filter(|element| element.scroll_panel == Some(panel))
            .map(|element| u32::from(element.rect[0]) + u32::from(element.rect[2]))
            .max()
            .expect("Recent activity authors scrolled content");
        assert!(
            rightmost <= content_width,
            "authored element right edge {rightmost} escapes content extent {content_width}",
        );
        assert!(
            content_width - rightmost <= TRAILING_SLACK_LOGICAL_POINTS,
            "content extent {content_width} is empty past {rightmost}",
        );
    }

    #[test]
    fn service_health_region_keeps_the_list_vertical() {
        let (content_width, content_height, region_width, region_height) =
            extent(DashboardScrollPanel::ServiceHealth);
        assert_eq!(
            content_width, region_width,
            "Service health must not gain inline travel",
        );
        assert_eq!(content_height - region_height, 75);
    }
}
