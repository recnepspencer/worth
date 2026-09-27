//! Every dashboard component that scrolls its content through a Mosaic
//! region: the page, whose rows can outgrow the stage, the two lists it
//! carries, and the review's changes, which a short viewport can crowd.
use worth_ui::facade::declaration::{ComponentId, ComponentViewportRegion};

use super::{container_component, page, review, DashboardScrollPanel};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DashboardScrollOwner {
    List(DashboardScrollPanel),
    Page,
    Review,
}

impl DashboardScrollOwner {
    /// The review's changes first, over everything its modal covers; then
    /// the lists, so each hit-tests in front of the page that carries it.
    pub const ALL: [Self; 4] = [
        Self::Review,
        Self::List(DashboardScrollPanel::ServiceHealth),
        Self::List(DashboardScrollPanel::RecentActivity),
        Self::Page,
    ];

    /// The layout container whose content the region scrolls.
    pub const fn container(self) -> &'static str {
        match self {
            Self::List(panel) => panel.owner(),
            Self::Page => page::PAGE,
            Self::Review => review::LIST,
        }
    }

    pub fn component(self) -> ComponentId {
        container_component(self.container())
    }

    pub const fn region(self) -> &'static str {
        match self {
            Self::List(panel) => panel.region(),
            Self::Page => "platform.pulse.mosaic.region.page",
            Self::Review => "platform.pulse.mosaic.region.review_list",
        }
    }

    /// Where the container places its region. The page's region is the
    /// stage beside the sidebar and below the masthead, which the page's own
    /// box outgrows when its rows' minimums exceed the stage.
    pub fn region_placement(self) -> ComponentViewportRegion {
        match self {
            Self::List(panel) => panel.region_placement(),
            Self::Page => page::stage(),
            Self::Review => review::list_region(),
        }
    }

    /// The declared distance a keyboard step or one wheel line moves this
    /// region, in logical points.
    pub const fn line_extent_logical_points(self) -> u16 {
        match self {
            Self::List(panel) => panel.line_extent_logical_points(),
            Self::Page => 40,
            Self::Review => 20,
        }
    }
}
