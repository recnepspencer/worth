#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRedoPlanningDenial {
    MalformedMember,
    WrongDomain,
    RecordCountLimit,
    TargetLimit,
    DistinctTargetLimit,
    RecoveryMemoryLimit { observed: u64, admitted: u64 },
    InvalidRecordOrder,
    NonCanonicalTargetOrder,
    LsnRangeMismatch,
    InvalidTarget,
    InvalidRecoveryProjection,
    MissingPageObservation,
    GenerationMismatch,
    PageDigestMismatch,
    ProvenNoEffectHasWalAttempt,
    CounterOverflow,
}
