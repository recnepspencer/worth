//! Pulse-owned product composition contracts staged for the 3.15 cutover.
//!
//! These values describe authored application composition. They are not a
//! renderer theme, framework appearance API, or evidence oracle.

mod mosaic;
mod runtime_services;
mod visual_composition;

#[doc(hidden)]
pub use mosaic::{
    PlatformPulseMosaicRegion, PlatformPulseMosaicSizing, PlatformPulseMosaicSurface,
    PLATFORM_PULSE_EVIDENCE_PLACEMENT, PLATFORM_PULSE_FOCUSED_REGION_STATE,
    PLATFORM_PULSE_SERVICE_PLACEMENT, PLATFORM_PULSE_STATUS_PLACEMENT,
};
#[doc(hidden)]
pub use runtime_services::{
    platform_pulse_portal_story_transition, PlatformPulseCommandStory,
    PlatformPulsePortalStoryTransition, PlatformPulseQueryDenialStory,
};
#[doc(hidden)]
pub use visual_composition::{
    PlatformPulseLogicalRect, PlatformPulsePaletteRole, PlatformPulseProductComponent,
    PlatformPulseProductFactSource, PlatformPulseProductRegion, PlatformPulseProductRegionContract,
    PlatformPulseProductTargetContract, PlatformPulseRgba, PlatformPulseServiceStoryGate,
    PlatformPulseSourceSignalRole, PlatformPulseStaticCopy, PlatformPulseTextRole,
    PlatformPulseTextStyleContract,
};

pub use visual_composition::{
    dashboard_containers, dashboard_elements, DashboardContainer, DashboardContent,
    DashboardElement, DashboardGraphic, DashboardLayoutCell, DashboardPlacement,
    DashboardScrollPanel, PLATFORM_PULSE_MASTHEAD_HEIGHT, PLATFORM_PULSE_SIDEBAR_WIDTH,
};
