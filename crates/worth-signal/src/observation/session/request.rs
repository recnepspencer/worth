#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignalObservationSurface {
    PerformedCounters,
    PerformedWork,
    DescriptiveLineage,
    DescriptiveFacts,
    FrontierTrace,
    ReplayDetail,
    OptionalTelemetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalObservationRequest {
    mask: u8,
}

impl SignalObservationRequest {
    const COUNTERS: u8 = 1;
    const WORK: u8 = 2;
    const LINEAGE: u8 = 4;
    const FACTS: u8 = 8;
    const FRONTIER: u8 = 16;
    const REPLAY: u8 = 32;
    const TELEMETRY: u8 = 64;
    const ALL: u8 = Self::COUNTERS
        | Self::WORK
        | Self::LINEAGE
        | Self::FACTS
        | Self::FRONTIER
        | Self::REPLAY
        | Self::TELEMETRY;

    pub const fn operation() -> Self {
        Self { mask: Self::ALL }
    }

    pub const fn empty() -> Self {
        Self { mask: 0 }
    }

    pub const fn counters() -> Self {
        Self {
            mask: Self::COUNTERS,
        }
    }

    pub const fn work() -> Self {
        Self { mask: Self::WORK }
    }

    pub const fn lineage() -> Self {
        Self {
            mask: Self::LINEAGE,
        }
    }

    pub const fn facts() -> Self {
        Self { mask: Self::FACTS }
    }

    pub const fn frontier() -> Self {
        Self {
            mask: Self::FRONTIER,
        }
    }

    pub const fn replay() -> Self {
        Self { mask: Self::REPLAY }
    }

    pub const fn telemetry() -> Self {
        Self {
            mask: Self::TELEMETRY,
        }
    }

    pub const fn with_performed_counters(self) -> Self {
        Self {
            mask: self.mask | Self::COUNTERS,
        }
    }

    pub const fn with_performed_work(self) -> Self {
        Self {
            mask: self.mask | Self::WORK,
        }
    }

    pub const fn with_descriptive_lineage(self) -> Self {
        Self {
            mask: self.mask | Self::LINEAGE,
        }
    }

    pub const fn with_descriptive_facts(self) -> Self {
        Self {
            mask: self.mask | Self::FACTS,
        }
    }

    pub const fn with_frontier_trace(self) -> Self {
        Self {
            mask: self.mask | Self::FRONTIER,
        }
    }

    pub const fn with_replay_detail(self) -> Self {
        Self {
            mask: self.mask | Self::REPLAY,
        }
    }

    pub const fn with_optional_telemetry(self) -> Self {
        Self {
            mask: self.mask | Self::TELEMETRY,
        }
    }

    pub const fn includes(self, surface: SignalObservationSurface) -> bool {
        self.mask & surface.bit() != 0
    }

    pub const fn is_empty(self) -> bool {
        self.mask == 0
    }

    pub(crate) const fn mask(self) -> u8 {
        self.mask
    }

    pub(crate) const fn from_mask(mask: u8) -> Self {
        Self { mask }
    }

    pub(crate) const fn default_continuous_mask() -> u8 {
        Self::ALL
    }
}

impl SignalObservationSurface {
    pub(crate) const fn bit(self) -> u8 {
        match self {
            Self::PerformedCounters => SignalObservationRequest::COUNTERS,
            Self::PerformedWork => SignalObservationRequest::WORK,
            Self::DescriptiveLineage => SignalObservationRequest::LINEAGE,
            Self::DescriptiveFacts => SignalObservationRequest::FACTS,
            Self::FrontierTrace => SignalObservationRequest::FRONTIER,
            Self::ReplayDetail => SignalObservationRequest::REPLAY,
            Self::OptionalTelemetry => SignalObservationRequest::TELEMETRY,
        }
    }
}
#[cfg(test)]
#[path = "request_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
