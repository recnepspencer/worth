#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulseMosaicRegion {
    Viewport,
    Masthead,
    EvidenceRail,
    ServiceStage,
    StatusBand,
    ServiceTile,
    NativeTile,
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
    Viewport,
    Masthead,
    EvidenceRail,
    ServiceStage,
    StatusBand,
}

impl PlatformPulseMosaicRegion {
    pub const ALL: [Self; 7] = [
        Self::Viewport,
        Self::Masthead,
        Self::EvidenceRail,
        Self::ServiceStage,
        Self::StatusBand,
        Self::ServiceTile,
        Self::NativeTile,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Viewport => "platform.pulse.mosaic.region.viewport",
            Self::Masthead => "platform.pulse.mosaic.region.masthead",
            Self::EvidenceRail => "platform.pulse.mosaic.region.evidence_rail",
            Self::ServiceStage => "platform.pulse.mosaic.region.service_stage",
            Self::StatusBand => "platform.pulse.mosaic.region.status_band",
            Self::ServiceTile => "platform.pulse.mosaic.region.service_tile",
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
    pub const ALL: [Self; 5] = [
        Self::Viewport,
        Self::Masthead,
        Self::EvidenceRail,
        Self::ServiceStage,
        Self::StatusBand,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Viewport => "platform.pulse.mosaic.sizing.viewport",
            Self::Masthead => "platform.pulse.mosaic.sizing.masthead",
            Self::EvidenceRail => "platform.pulse.mosaic.sizing.evidence_rail",
            Self::ServiceStage => "platform.pulse.mosaic.sizing.service_stage",
            Self::StatusBand => "platform.pulse.mosaic.sizing.status_band",
        }
    }

    pub const fn named_measurement(self) -> Option<(&'static str, u32)> {
        match self {
            Self::Viewport | Self::ServiceStage => None,
            Self::Masthead => Some(("platform.pulse.measurement.masthead_height", 56)),
            Self::EvidenceRail => Some(("platform.pulse.measurement.evidence_width", 216)),
            Self::StatusBand => Some(("platform.pulse.measurement.status_height", 24)),
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
