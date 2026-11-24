use crate::publication::patch::data::PatchStreamPosition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriberRecoverySource {
    InMemoryHistory,
    DurableCanonicalRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriberRecoveryDisposition {
    StartFromBeginning,
    ResumeAfterCheckpoint,
    ContinueWithTransparentBridge,
    ContinueWithVisibleBridge,
    ContinueWithContractUpgrade,
    RequireRenegotiation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriberRecoveryDecision {
    pub disposition: SubscriberRecoveryDisposition,
    pub source: SubscriberRecoverySource,
    pub start_after_position: Option<PatchStreamPosition>,
}
