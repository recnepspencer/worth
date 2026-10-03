#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectExecutionDeferredKind {
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    TransactionRetentionCapacityExhausted,
    PatchPositionReservationContended,
    RetentionBackpressure,
    CandidateLifetimeExpired {
        maximum_lifetime_millis: u64,
    },
    CandidateCapacityExhausted {
        maximum_candidates: usize,
    },
    PublishedSnapshotCapacityExhausted {
        maximum_handles: usize,
    },
    /// Query's invalidation index has not yet accepted this branch's
    /// commit; the effect may be retried unchanged.
    InvalidationCompanionPending,
}
