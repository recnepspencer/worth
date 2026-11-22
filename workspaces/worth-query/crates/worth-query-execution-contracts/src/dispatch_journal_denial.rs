//! Durable recovery claim and dispatch-journal denial vocabulary.

use worth_foundational::facade::{
    CorrectionMaterialPinIdentity, DurableCompositeCommitIdentity,
    DurableCompositePublicationAttemptIdentity, DurableRecoveryClaimIdentity,
    RecoveryClaimDisposition,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryRecoveryClaim {
    pub original: DurableCompositeCommitIdentity,
    pub attempt: DurableCompositePublicationAttemptIdentity,
    pub expires_at_unix_ms: u64,
    pub material_pin: CorrectionMaterialPinIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryRecoveryRecord {
    Claim(QueryRecoveryClaim),
    Disposition {
        claim: DurableRecoveryClaimIdentity,
        terminal: RecoveryClaimDisposition,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryRecordedRecoveryClaim {
    pub identity: DurableRecoveryClaimIdentity,
    pub claim: QueryRecoveryClaim,
    pub terminal: Option<RecoveryClaimDisposition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryDispatchJournalDenial {
    UnsupportedVersion(u32),
    MalformedFrame,
    Checksum,
    ForeignHome,
    PrefixMismatch,
    IdentityExhausted,
    AttemptIdentityMismatch,
    /// The same effect already has an attempt awaiting its observation.
    AttemptAwaitingObservation,
    /// A completed external effect is never contacted again.
    EffectAlreadyCompleted,
    UnknownAttempt,
    AttemptAlreadyObserved,
    /// This runtime has no durable Home journal; its custody is process-local.
    JournalUnavailable,
    OwnerUnavailable,
    StaleIncarnation,
    PublicationPending,
    PublicationNoEffect,
    PublicationGenerationMismatch,
    PublicationUnresolved,
    Storage(String),
    Execution(worth_execution::WorkCeilingDenial),
}
