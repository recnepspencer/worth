//! Process-local durability postures carried by execution outcomes.

/// How durable a recovery handle is, as stated on its inspection view.
///
/// Recovery handles live in process memory. Surviving a restart needs a store
/// capability, which the posture names instead of implying durability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRecoveryDurabilityPosture {
    /// The handle is process-local; durable survival requires a store capability.
    StoreCapabilityRequired,
}

impl WorthQueryRecoveryDurabilityPosture {
    pub const fn as_decision58_label(self) -> &'static str {
        match self {
            Self::StoreCapabilityRequired => "store-capability-required",
        }
    }
}

/// How durable the dispatch outbox is, as stated on a safe-retry admission.
///
/// The outbox record lives in process memory. Surviving a restart needs a store
/// capability, which the posture names instead of implying durability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDispatchOutboxDurabilityPosture {
    /// The outbox is process-local; durable survival requires a store capability.
    StoreCapabilityRequired,
}

impl WorthQueryDispatchOutboxDurabilityPosture {
    pub const fn as_decision58_label(self) -> &'static str {
        match self {
            Self::StoreCapabilityRequired => "store-capability-required",
        }
    }
}
