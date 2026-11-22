//! Core provider-session protocol denial, stop, stage, and recovery posture enums.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionRecoveryPosture {
    Closed,
    RecoveryRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionProtocolStage {
    PlanAdmission,
    PlanReadmission,
    SessionPreparation,
    StagedPreparation,
    Commit,
    Abort,
}

/// The request control condition that stopped provider execution before publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionControlStopKind {
    Cancelled,
    TimedOut,
}

/// A provider-session refusal that preserves its protocol or execution cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionDenialKind {
    /// Retained content disagreed with its certified identity.
    ContentInconsistency,
    /// Exact Native content refusal, distinct from request/resource stops.
    Content(worth_relational::facade::content::RelationalContentRejection),
    /// The selected root refused the requested projection contract.
    ProjectionContract {
        denial: worth_relational::facade::runtime::RelationalProjectionContractDenial,
        partition_identity: Option<u64>,
    },
    /// Retained history could not supply the requested visibility basis.
    HistoricalVisibility {
        denial: worth_relational::facade::runtime::HistoricalVisibilityDenial,
        partition_identity: Option<u64>,
    },
    StorageUnavailable(worth_relational::facade::storage::RelationalStorageUnavailable),
    ForeignOperationAttempt,
    ForeignExecutionBasis,
    ForeignGraphAuthority,
    StaleIncarnation,
    UndeclaredOperationScope,
    ResourceEnvelopeMismatch,
    /// An execution refusal retains Query's shared resource vocabulary.
    ExecutionResource {
        denial: crate::computation_denial::WorthQueryManagedComputationResourceDenial,
        partition_identity: Option<u64>,
        /// The refusing ancestor, only for a policy memory reservation.
        policy_ancestor: Option<u32>,
    },
    ExecutionNestedPatternStopped {
        partition_identity: Option<u64>,
    },
    ExecutionWorkerPanicked {
        partition_identity: Option<u64>,
    },
    ExecutionIdentitiesNotCanonical {
        partition_identity: Option<u64>,
    },
    /// A custom invariant declined the checked preparation contract.
    ExecutionUncheckedCustomKernel {
        partition_identity: Option<u64>,
    },
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    CandidateIdentityExhausted,
    IndexMaintenanceBudgetExceeded,
    IndexGenerationIdentityExhausted,
    ProviderIdentityMismatch,
    ProviderGenerationMismatch,
    SessionProtocolUnsupported,
    AllocationDenied,
    ProviderRejected,
    /// Canonical output membership disagreed before the World transition.
    OutputCorrespondenceMismatch {
        cause: crate::output_projection_denial::WorthQueryApplicationOutputProjectionDenial,
    },
    ProviderPanicked,
    TokenNotMintedForPlan,
    EmptyPhysicalSessionIdentity,
    SessionIdentityExhausted,
}
