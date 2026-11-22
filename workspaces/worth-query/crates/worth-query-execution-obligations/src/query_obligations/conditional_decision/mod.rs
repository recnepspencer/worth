//! Durable continuation meaning; Relational owns input and World owns effects.
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableProductBranchIncarnation,
    ManagedCleanupObligationIdentity,
};

/// A retry needs its original key and intent, not the live error or clock lease.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryConditionalPosture {
    CleanupRequired(ManagedCleanupObligationIdentity),
    Retryable,
    CommitRetryable,
    ProductStale,
    /// R9 `OperationIndeterminate`: Query lost the commit's answer, so World's
    /// state for the original key decides its fate after complete readmission.
    Indeterminate(QueryConditionalOriginal),
}

/// The World original an indeterminate decision belongs to.
///
/// The live owner never learns it (the answer was lost); only the cold
/// consumer joins a row to World's recovered unpublished original, and a
/// joined row never moves to another original.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryConditionalOriginal {
    /// Recorded by the live owner; World has not classified the key yet.
    Unclassified,
    /// Joined, before service, to World's recovered unpublished original.
    Recovered(DurableCompositePublicationAttemptIdentity),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryConditionalDecision {
    pub key: [u8; 32],
    pub intent: [u8; 32],
    /// The durable product branch occurrence where this key was derived and
    /// where World resolves its fate.
    pub product: DurableProductBranchIncarnation,
    pub posture: QueryConditionalPosture,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryConditionalDecisionRecord {
    Retain(QueryConditionalDecision),
    Finish { key: [u8; 32], intent: [u8; 32] },
}

pub(super) mod codec;
mod fold;
