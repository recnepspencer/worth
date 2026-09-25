use worth_ui::facade::declaration::{ComponentViewportAxisPlacement, ComponentViewportRegion};

use super::{PLATFORM_PULSE_MASTHEAD_HEIGHT, PLATFORM_PULSE_SIDEBAR_WIDTH};

/// The status band's height, closing the bottom of the rail.
const STATUS_BAND_HEIGHT: u16 = 94;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulseMosaicRegion {
    Viewport,
    Masthead,
    EvidenceRail,
    ServiceStage,
    StatusBand,
    ServiceTile,
    NativeTile,
    ServiceList,
    ActivityList,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulseMosaicSurface {
    Main,
    Evidence,
    Service,
    Status,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulseMosaicSizing {
    DashboardList,
    Viewport,
    Masthead,
    EvidenceRail,
    ServiceStage,
    StatusBand,
}

impl PlatformPulseMosaicRegion {
    pub const ALL: [Self; 9] = [
        Self::Viewport,
        Self::Masthead,
        Self::EvidenceRail,
        Self::ServiceStage,
        Self::StatusBand,
        Self::ServiceTile,
        Self::NativeTile,
        Self::ServiceList,
        Self::ActivityList,
    ];

    /// The regions the dashboard surface owner places across the viewport.
    pub const SURFACE: [Self; 5] = [
        Self::Viewport,
        Self::Masthead,
        Self::EvidenceRail,
        Self::ServiceStage,
        Self::StatusBand,
    ];

    /// Where the surface owner places this region across the viewport: the
    /// rail runs the full height on the left, the masthead spans the rest of
    /// the top, the service stage fills beneath it, and the status band closes
    /// the rail. `None` for a region another owner places.
    pub fn surface_placement(self) -> Option<ComponentViewportRegion> {
        let fill = ComponentViewportAxisPlacement::stretch_between(0, 0);
        let rail =
            ComponentViewportAxisPlacement::fixed_from_start(0, PLATFORM_PULSE_SIDEBAR_WIDTH)
                .expect("the evidence rail has width");
        let beside_rail =
            ComponentViewportAxisPlacement::stretch_between(PLATFORM_PULSE_SIDEBAR_WIDTH, 0);
        let (horizontal, vertical) = match self {
            Self::Viewport => (fill, fill),
            Self::Masthead => (
                beside_rail,
                ComponentViewportAxisPlacement::fixed_from_start(0, PLATFORM_PULSE_MASTHEAD_HEIGHT)
                    .expect("the masthead has height"),
            ),
            Self::EvidenceRail => (rail, fill),
            Self::ServiceStage => (
                beside_rail,
                ComponentViewportAxisPlacement::stretch_between(PLATFORM_PULSE_MASTHEAD_HEIGHT, 0),
            ),
            Self::StatusBand => (
                rail,
                ComponentViewportAxisPlacement::fixed_from_end(0, STATUS_BAND_HEIGHT)
                    .expect("the status band has height"),
            ),
            Self::ServiceTile | Self::NativeTile | Self::ServiceList | Self::ActivityList => {
                return None;
            }
        };
        Some(ComponentViewportRegion::new(horizontal, vertical))
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Viewport => "platform.pulse.mosaic.region.viewport",
            Self::Masthead => "platform.pulse.mosaic.region.masthead",
            Self::EvidenceRail => "platform.pulse.mosaic.region.evidence_rail",
            Self::ServiceStage => "platform.pulse.mosaic.region.service_stage",
            Self::StatusBand => "platform.pulse.mosaic.region.status_band",
            Self::ServiceTile => "platform.pulse.mosaic.region.service_tile",
            Self::ServiceList => "platform.pulse.mosaic.region.service_list",
            Self::ActivityList => "platform.pulse.mosaic.region.activity_list",
            Self::NativeTile => "platform.pulse.mosaic.region.native_tile",
        }
    }
}

impl PlatformPulseMosaicSurface {
    pub const ALL: [Self; 4] = [Self::Main, Self::Evidence, Self::Service, Self::Status];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Main => "platform.pulse.surface.main",
            Self::Evidence => "platform.pulse.surface.evidence",
            Self::Service => "platform.pulse.surface.service",
            Self::Status => "platform.pulse.surface.status",
        }
    }
}

impl PlatformPulseMosaicSizing {
    pub const ALL: [Self; 6] = [
        Self::DashboardList,
        Self::Viewport,
        Self::Masthead,
        Self::EvidenceRail,
        Self::ServiceStage,
        Self::StatusBand,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::DashboardList => "platform.pulse.mosaic.sizing.dashboard_list",
            Self::Viewport => "platform.pulse.mosaic.sizing.viewport",
            Self::Masthead => "platform.pulse.mosaic.sizing.masthead",
            Self::EvidenceRail => "platform.pulse.mosaic.sizing.evidence_rail",
            Self::ServiceStage => "platform.pulse.mosaic.sizing.service_stage",
            Self::StatusBand => "platform.pulse.mosaic.sizing.status_band",
        }
    }

    pub const fn named_measurement(self) -> Option<(&'static str, u32)> {
        match self {
            Self::DashboardList | Self::Viewport | Self::ServiceStage => None,
            Self::Masthead => Some((
                "platform.pulse.measurement.masthead_height",
                PLATFORM_PULSE_MASTHEAD_HEIGHT as u32,
            )),
            Self::EvidenceRail => Some((
                "platform.pulse.measurement.evidence_width",
                PLATFORM_PULSE_SIDEBAR_WIDTH as u32,
            )),
            Self::StatusBand => Some((
                "platform.pulse.measurement.status_height",
                STATUS_BAND_HEIGHT as u32,
            )),
        }
    }
}

pub const PLATFORM_PULSE_FOCUSED_REGION_STATE: &str =
    "platform.pulse.mosaic.state.focused_service_region";
pub const PLATFORM_PULSE_EVIDENCE_PLACEMENT: &str = "platform.pulse.mosaic.placement.evidence";
pub const PLATFORM_PULSE_SERVICE_PLACEMENT: &str = "platform.pulse.mosaic.placement.service";
pub const PLATFORM_PULSE_STATUS_PLACEMENT: &str = "platform.pulse.mosaic.placement.status";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mosaic_contract_keeps_product_region_identities_distinct() {
        let mut identities = PlatformPulseMosaicRegion::ALL
            .map(PlatformPulseMosaicRegion::id)
            .to_vec();
        identities.sort_unstable();
        identities.dedup();
        assert_eq!(identities.len(), PlatformPulseMosaicRegion::ALL.len());
    }
}
