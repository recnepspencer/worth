mod component_identity;
mod copy;
mod dashboard;
mod geometry;
mod palette;
mod projection;
mod typography;
pub use dashboard::{
    dashboard_containers, dashboard_elements, DashboardContainer, DashboardContent,
    DashboardElement, DashboardGraphic, DashboardLayoutCell, DashboardPlacement,
    DashboardScrollOwner, DashboardScrollPanel, PLATFORM_PULSE_MASTHEAD_HEIGHT,
    PLATFORM_PULSE_SIDEBAR_WIDTH,
};

pub use component_identity::PlatformPulseProductComponent;
pub use copy::PlatformPulseStaticCopy;
pub use geometry::PlatformPulseLogicalRect;
pub use palette::{PlatformPulsePaletteRole, PlatformPulseRgba, PlatformPulseSourceSignalRole};
pub use projection::{
    PlatformPulseProductFactSource, PlatformPulseProductRegion, PlatformPulseProductRegionContract,
    PlatformPulseProductTargetContract, PlatformPulseServiceStoryGate,
};
pub use typography::{PlatformPulseTextRole, PlatformPulseTextStyleContract};
