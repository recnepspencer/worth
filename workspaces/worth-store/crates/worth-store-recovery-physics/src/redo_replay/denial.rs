#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRedoPlanningDenial {
    MalformedMember,
    WrongDomain,
    RecordCountLimit,
    TargetLimit,
    DistinctTargetLimit,
    RecoveryMemoryLimit {
        observed: u64,
        admitted: u64,
    },
    InvalidRecordOrder,
    NonCanonicalTargetOrder,
    LsnRangeMismatch,
    InvalidTarget,
    InvalidRecoveryProjection,
    UnsupportedRecoveryProjectionVersion(u16),
    MissingPageObservation,
    GenerationMismatch,
    PageDigestMismatch,
    ProvenNoEffectHasWalAttempt,
    CounterOverflow,
    /// A terminal head retirement member was admitted by C.9 but cannot yet
    /// be planned.
    TerminalHeadRetirementUnsupported,
}
