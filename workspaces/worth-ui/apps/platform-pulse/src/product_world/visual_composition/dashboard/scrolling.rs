//! Authored content extents for the two independent dashboard lists.
//!
//! This file authors content only. A panel's viewport is the allocation of its
//! own Mosaic region and is never restated here, so content that overflows is
//! overflow the region can actually prove.
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
}
