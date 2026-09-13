//! Authored content and viewport extents for the two independent dashboard lists.
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
    pub const fn content_rect(self) -> [u16; 4] {
        match self {
            Self::ServiceHealth => [1120, 324, 360, 344],
            Self::RecentActivity => [290, 693, 768, 336],
        }
    }
    pub const fn viewport_height(self) -> u16 {
        match self {
            Self::ServiceHealth => 269,
            Self::RecentActivity => 269,
        }
    }
    pub const fn region(self) -> &'static str {
        match self {
            Self::ServiceHealth => "platform.pulse.mosaic.region.service_list",
            Self::RecentActivity => "platform.pulse.mosaic.region.activity_list",
        }
    }
}
