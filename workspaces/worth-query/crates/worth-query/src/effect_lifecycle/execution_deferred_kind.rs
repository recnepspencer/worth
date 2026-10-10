use worth_relational::facade::mvcc::CompanionPreflightStop;

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
    /// commit. Resubmitting the same effect prepares it again against the
    /// current head, which may then report a stale or changed basis.
    InvalidationCompanionPending,
    /// Query's invalidation index reached an installed limit while preparing
    /// this commit. An unchanged retry meets the same limit until capacity is
    /// released or the installation is enlarged.
    InvalidationCompanionCapacityExhausted,
}

/// Whether an invalidation companion stop clears without any change to the
/// effect: registration, topology contention, or a selected root or position
/// moved by a concurrent publication. Budgets, capacities, counter overflow and
/// a cell owned by another runtime or binding recur.
pub(crate) const fn companion_stop_is_transient(stop: &CompanionPreflightStop) -> bool {
    match stop {
        CompanionPreflightStop::TopologyPending
        | CompanionPreflightStop::SelectedSourceMismatch
        | CompanionPreflightStop::SelectedPositionUnavailable { .. }
        | CompanionPreflightStop::RegistrationChanged => true,
        // Publication reports interruption as its own outcome before a
        // companion deferral is built, so it never reaches this split.
        CompanionPreflightStop::Interrupted(_)
        | CompanionPreflightStop::ForeignCell
        | CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow
        | CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow
        | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. } => false,
    }
}
