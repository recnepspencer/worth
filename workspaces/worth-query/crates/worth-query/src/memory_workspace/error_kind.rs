use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryWorkspaceErrorKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Unclassified,
    ForeignPhase(
        worth_query_execution::facade::application_contribution::WorthQueryForeignAdvancementPhase,
    ),
    /// The execution owner refused an operation before any effect.
    ExecutionDenied(
        worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind,
    ),
    /// The execution owner canceled or timed out before any effect.
    ExecutionControlStopped(
        worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind,
    ),
    UnsupportedCollection,
    UnsupportedWriteFamily,
    BatchAtomicityUnsupported,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    TransactionAllocationDenied {
        kind: worth_execution::ExecutionAllocationDenialClass,
        requested_payload_bytes: Option<u64>,
    },
    TransactionStagingCardinalityOverflow,
    TransactionSymbolIdentityExhausted,
    TransactionInputDirectoryAllocationDenied {
        requested_batches: usize,
    },
    SavepointCapacityExhausted {
        maximum_savepoints: usize,
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
    PatchPositionReservationContended,
    ProposalIdentityExhausted,
    RelationalBasisUnavailable,
    InvalidationCompanionPending,
    InvalidationCompanionCapacityExhausted,
    /// Structural managed Source stop, with no retry or capacity inference.
    InvalidationCompanionUnavailable {
        stop: worth_relational::facade::mvcc::CompanionPreflightStop,
    },
    /// The durable medium could not say whether the commit became durable. It
    /// was not installed live and may already be durable; the Relational owner
    /// refuses further publication until it is reopened.
    DurabilityUnresolved,
}
