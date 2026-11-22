//! Pure runtime-boundary denial enums.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchAdmissionDenial {
    ForeignRelationalRuntime,
    ForeignProductWorld,
    PublicationCommitMismatch,
    MissingInboundOperationSlot,
    AlreadyCompleted,
    CompletedTransportRetained,
    TerminalIndexUnavailable,
    AttemptIdentityExhausted,
    DurableAttemptRecordRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstalledTransportPendingReason {
    ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    UnknownCompletion,
    ConcurrentContinuation,
    PublicationRetryRequired,
    AllocationDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: worth_execution::ExecutionAllocationDenialClass,
        requested_payload_bytes: Option<u64>,
    },
    StagingCardinalityOverflow,
    InputDirectoryAllocationDenied {
        requested_batches: usize,
    },
    ExecutionDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: crate::domain_computation::WorthQueryProviderSessionDenialKind,
    },
    ExecutionControlStopped {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    },
    PublicationAtCapacity,
    ProductRecoveryRequired,
    RecoveryStaleProduct,
    TerminalProtectionUnavailable,
    TerminalReleaseUnavailable,
}
