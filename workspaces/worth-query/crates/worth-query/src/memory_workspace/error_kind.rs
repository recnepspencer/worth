#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryWorkspaceErrorKind {
    Unclassified,
    /// The execution owner refused a commit before any effect.
    ExecutionDenied(
        worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind,
    ),
    /// The execution owner canceled or timed out before any effect.
    ExecutionControlStopped(
        worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind,
    ),
    UnsupportedCollection,
    UnsupportedWriteFamily,
    EmptySchema,
    BatchAtomicityUnsupported,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    TransactionOverlayCapacityExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    TransactionFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    SavepointCapacityExhausted {
        maximum_savepoints: usize,
    },
    SavepointFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    SavepointIdentityExhausted,
    TransactionMaterializationAuthorityRequired,
    TransactionMaterializationModeMismatch,
    CandidateCapacityExhausted {
        maximum_candidates: usize,
    },
    PublishedSnapshotCapacityExhausted {
        maximum_handles: usize,
    },
    CandidateIdentityExhausted,
    PreparedRootBudgetExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    PatchPositionReservationContended,
    ProposalIdentityExhausted,
    RelationalBasisUnavailable,
    InvalidationCompanionPending,
    InvalidationCompanionCapacityExhausted,
}
