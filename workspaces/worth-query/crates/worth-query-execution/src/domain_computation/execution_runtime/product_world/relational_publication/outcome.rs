use super::WorthQueryPerformedRelationalProductChange;

/// Delivery posture for a fresh World publication.
///
/// Success consumes the performed-change authority. Every other posture
/// returns that exact move-only authority so the caller can retry it.
#[derive(Debug)]
pub enum WorthQueryPerformedRelationalProductChangeDeliveryOutcome {
    Success(worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt),
    Denied {
        posture: worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryDenial,
        change: WorthQueryPerformedRelationalProductChange,
    },
    Deferred {
        posture: worth_runtime_bridge::facade::BridgeCorrespondenceDeferred,
        change: WorthQueryPerformedRelationalProductChange,
    },
    Stale {
        posture: worth_runtime_bridge::facade::BridgeCorrespondenceStale,
        change: WorthQueryPerformedRelationalProductChange,
    },
    RebindRequired {
        posture: worth_runtime_bridge::facade::BridgeCorrespondenceRebindRequired,
        change: WorthQueryPerformedRelationalProductChange,
    },
    Failed {
        posture: worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionFailure,
        change: WorthQueryPerformedRelationalProductChange,
    },
}

impl WorthQueryPerformedRelationalProductChangeDeliveryOutcome {
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn into_retry_change(self) -> Option<WorthQueryPerformedRelationalProductChange> {
        match self {
            Self::Success(_) => None,
            Self::Denied { change, .. }
            | Self::Deferred { change, .. }
            | Self::Stale { change, .. }
            | Self::RebindRequired { change, .. }
            | Self::Failed { change, .. } => Some(change),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryPerformedRelationalProductChangeDeliveryDenialKind {
    ForeignProductRoot,
    Bridge,
}

/// A denied delivery returns the unique performed witness to its caller.
#[derive(Debug)]
pub struct WorthQueryPerformedRelationalProductChangeDeliveryDenial {
    kind: WorthQueryPerformedRelationalProductChangeDeliveryDenialKind,
    detail: String,
    change: WorthQueryPerformedRelationalProductChange,
}

impl WorthQueryPerformedRelationalProductChangeDeliveryDenial {
    pub(crate) fn new(
        kind: WorthQueryPerformedRelationalProductChangeDeliveryDenialKind,
        detail: impl Into<String>,
        change: WorthQueryPerformedRelationalProductChange,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            change,
        }
    }

    pub const fn kind(&self) -> WorthQueryPerformedRelationalProductChangeDeliveryDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn into_change(self) -> WorthQueryPerformedRelationalProductChange {
        self.change
    }
}
