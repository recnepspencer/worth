//! Moved whole from `domain_computation::primary_graph::provider::committed_dispatch_outbox`: the items below and every impl they own.

/// Why Query could not establish an authoritative committed outbox row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCommittedDispatchOutboxReadDenial {
    ReadExecution(
        Result<
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    ForeignPhase,
    ForeignRuntime,
    Missing,
    AmbiguousCorrelation,
    PendingPublication,
    CommittedIndexUnavailable,
    WrongRecordKind,
    NotAuthoritative,
    ExactCommitUnavailable,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    SnapshotIdentityExhausted,
    Malformed,
    CommitMismatch,
    RecordMismatch,
}
