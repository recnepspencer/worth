//! Durable continuation meaning; Relational owns input and World owns effects.
use worth_foundational::facade::ManagedCleanupObligationIdentity;

/// A retry needs its original key and intent, not the live error or clock lease.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryConditionalPosture {
    CleanupRequired(ManagedCleanupObligationIdentity),
    Retryable,
    CommitRetryable,
    ProductStale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryConditionalDecision {
    pub key: [u8; 32],
    pub intent: [u8; 32],
    pub posture: QueryConditionalPosture,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryConditionalDecisionRecord {
    Retain(QueryConditionalDecision),
    Finish { key: [u8; 32], intent: [u8; 32] },
}

pub(super) mod codec;
mod fold;
