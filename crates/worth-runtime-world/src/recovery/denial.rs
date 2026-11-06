/// Truthful denial from the owning recovery catalog or its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldRecoveryDenial {
    HomeProgramTransitionRequired,
    RecoveredCustodyRequired,
    ContinuationAlreadyCommitted,
    ContinuationAlreadyStarted,
    ReleaseAlreadyStarted,
    ReopenRequired,
    OwnerUnavailable(crate::lifecycle::RuntimeWorldOwnerUnavailable),
    ForeignHandle,
    StaleIncarnation,
    MissingRecord,
    Busy,
    TooYoung,
    ClockRegressed,
    CallerCapabilityLive,
    SettlementRequired,
    SettlementEvidenceUnavailable,
    OriginalSourceUnavailable,
    OriginalSourceMismatch,
    OutputCapacityExhausted,
}
impl From<crate::lifecycle::RuntimeWorldOwnerUnavailable> for RuntimeWorldRecoveryDenial {
    fn from(value: crate::lifecycle::RuntimeWorldOwnerUnavailable) -> Self {
        Self::OwnerUnavailable(value)
    }
}
