//! Host-surface bounds of the dashboard scroll regions, resolved from each
//! region's own authored allocation.
//!
//! A scroll region's viewport is its own rectangle. It is never reconstructed
//! from the content owner it clips, so the content extents authored in
//! `product_world` are free to exceed it on either axis.
use worth_ui::facade::app::{UiMountedCanonicalBox, UiMountedCoordinateSpace};
use worth_ui::facade::declaration::ComponentAllocationMeasurementContract;
use worth_ui_platform_pulse::product_world::{DashboardScrollPanel, PlatformPulseLogicalRect};

/// The region's own bounds when `region_kind` names a dashboard scroll region,
/// or `None` when the surface allocates that region instead.
pub(super) fn scroll_region_surface_bounds(
    region_kind: &str,
    viewport: UiMountedCanonicalBox,
) -> Result<Option<UiMountedCanonicalBox>, String> {
    let Some(panel) = allocated_scroll_region(region_kind) else {
        return Ok(None);
    };
    super::resolve_allocation(
        region_allocation(panel),
        viewport,
        UiMountedCoordinateSpace::HostSurface,
    )
    .map(Some)
}

fn allocated_scroll_region(region_kind: &str) -> Option<DashboardScrollPanel> {
    DashboardScrollPanel::ALL
        .into_iter()
        .find(|panel| panel.region() == region_kind)
}

fn region_allocation(panel: DashboardScrollPanel) -> ComponentAllocationMeasurementContract {
    let [x, y, width, height] = authored_region_rect(panel);
    PlatformPulseLogicalRect::new(x.into(), y.into(), width.into(), height.into()).allocation()
}

/// The authored viewport rectangle of each dashboard scroll region, in
/// host-surface logical points. Service health keeps its region as wide as its
/// content, so it stays a vertical list; Recent activity's region is narrower
/// than its content, so the list travels on both axes.
const fn authored_region_rect(panel: DashboardScrollPanel) -> [u16; 4] {
    match panel {
        DashboardScrollPanel::ServiceHealth => [1_120, 324, 360, 269],
        DashboardScrollPanel::RecentActivity => [290, 693, 768, 269],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_platform_pulse::product_world::dashboard_elements;

    /// The horizontal slack the authored content may leave beyond its last
    /// element before the overflow stops being real content.
    const TRAILING_SLACK_LOGICAL_POINTS: u32 = 32;

    fn extent(panel: DashboardScrollPanel) -> (u32, u32, u32, u32) {
        let [_, _, content_width, content_height] = panel.content_rect();
        let [_, _, region_width, region_height] = authored_region_rect(panel);
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
