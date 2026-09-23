#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForegroundLatencyEnvelopeKind {
    HardBound,
    SoftSlo,
    BoundedInterference,
    StarvationFreedom,
    CertificationOnlyTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForegroundLatencyEnvelope {
    kind: ForegroundLatencyEnvelopeKind,
    profile_scope: &'static str,
    max_interference_events: Option<u64>,
}

impl ForegroundLatencyEnvelope {
    pub const fn hard_bound(profile_scope: &'static str, max_interference_events: u64) -> Self {
        Self {
            kind: ForegroundLatencyEnvelopeKind::HardBound,
            profile_scope,
            max_interference_events: Some(max_interference_events),
        }
    }

    pub const fn soft_slo(profile_scope: &'static str, max_interference_events: u64) -> Self {
        Self {
            kind: ForegroundLatencyEnvelopeKind::SoftSlo,
            profile_scope,
            max_interference_events: Some(max_interference_events),
        }
    }

    pub const fn bounded_interference(
        profile_scope: &'static str,
        max_interference_events: u64,
    ) -> Self {
        Self {
            kind: ForegroundLatencyEnvelopeKind::BoundedInterference,
            profile_scope,
            max_interference_events: Some(max_interference_events),
        }
    }

    pub const fn starvation_freedom(profile_scope: &'static str) -> Self {
        Self {
            kind: ForegroundLatencyEnvelopeKind::StarvationFreedom,
            profile_scope,
            max_interference_events: None,
        }
    }

    pub const fn certification_only_target(profile_scope: &'static str) -> Self {
        Self {
            kind: ForegroundLatencyEnvelopeKind::CertificationOnlyTarget,
            profile_scope,
            max_interference_events: None,
        }
    }

    pub const fn claims_unprovided_service_time(self) -> bool {
        matches!(
            self.kind,
            ForegroundLatencyEnvelopeKind::HardBound | ForegroundLatencyEnvelopeKind::SoftSlo
        )
    }

    pub const fn kind(self) -> ForegroundLatencyEnvelopeKind {
        self.kind
    }

    pub const fn profile_scope(self) -> &'static str {
        self.profile_scope
    }

    pub const fn max_interference_events(self) -> Option<u64> {
        self.max_interference_events
    }
}
