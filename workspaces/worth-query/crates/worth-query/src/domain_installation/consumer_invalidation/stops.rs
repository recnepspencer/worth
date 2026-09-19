#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConsumerInvalidationDeltaStopKind {
    ForeignOrStaleLease,
    ConsumerSupportUnavailable,
    NoSemanticDelivery,
    ImpactDeliveryMismatch,
}

#[derive(Debug)]
pub struct WorthQueryConsumerInvalidationDeltaStop {
    kind: WorthQueryConsumerInvalidationDeltaStopKind,
    counters: super::WorthQueryConsumerInvalidationCounters,
    epoch_counters: super::WorthQueryConsumerInvalidationEpochCounters,
    support: Option<(
        crate::domain_installation::WorthQueryConsumerSupportDimension,
        crate::domain_installation::WorthQueryConsumerSupportPosture,
    )>,
}

pub struct WorthQueryConsumerInvalidationAdmissionStop {
    pub(super) kind: WorthQueryConsumerInvalidationDeltaStopKind,
    pub(super) delta: super::WorthQueryConsumerInvalidationDelta,
    pub(super) counters: super::WorthQueryConsumerInvalidationCounters,
}

impl WorthQueryConsumerInvalidationAdmissionStop {
    pub const fn kind(&self) -> WorthQueryConsumerInvalidationDeltaStopKind {
        self.kind
    }

    pub fn into_delta(self) -> super::WorthQueryConsumerInvalidationDelta {
        self.delta
    }

    pub const fn counters(&self) -> super::WorthQueryConsumerInvalidationCounters {
        self.counters
    }
}

impl WorthQueryConsumerInvalidationDeltaStop {
    pub(super) fn new(
        kind: WorthQueryConsumerInvalidationDeltaStopKind,
        counters: super::WorthQueryConsumerInvalidationCounters,
        epoch_counters: super::WorthQueryConsumerInvalidationEpochCounters,
    ) -> Self {
        Self {
            kind,
            counters,
            epoch_counters,
            support: None,
        }
    }

    pub(super) fn unsupported(
        dimension: crate::domain_installation::WorthQueryConsumerSupportDimension,
        posture: crate::domain_installation::WorthQueryConsumerSupportPosture,
        counters: super::WorthQueryConsumerInvalidationCounters,
        epoch_counters: super::WorthQueryConsumerInvalidationEpochCounters,
    ) -> Self {
        Self {
            kind: WorthQueryConsumerInvalidationDeltaStopKind::ConsumerSupportUnavailable,
            counters,
            epoch_counters,
            support: Some((dimension, posture)),
        }
    }

    pub const fn kind(&self) -> WorthQueryConsumerInvalidationDeltaStopKind {
        self.kind
    }

    pub const fn counters(&self) -> super::WorthQueryConsumerInvalidationCounters {
        self.counters
    }

    pub const fn epoch_counters(&self) -> super::WorthQueryConsumerInvalidationEpochCounters {
        self.epoch_counters
    }

    pub const fn support_dimension(
        &self,
    ) -> Option<crate::domain_installation::WorthQueryConsumerSupportDimension> {
        match self.support {
            Some((dimension, _)) => Some(dimension),
            None => None,
        }
    }

    pub const fn support_posture(
        &self,
    ) -> Option<crate::domain_installation::WorthQueryConsumerSupportPosture> {
        match self.support {
            Some((_, posture)) => Some(posture),
            None => None,
        }
    }
}
