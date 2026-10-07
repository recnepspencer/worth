use super::WorthQueryProviderSessionProtocolCounters;

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

/// A provider-session refusal that preserves its protocol or execution cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionDenialKind {
    ForeignOperationAttempt,
    ForeignExecutionBasis,
    ForeignGraphAuthority,
    UndeclaredOperationScope,
    ResourceEnvelopeMismatch,
    /// An execution refusal retains Query's shared resource vocabulary.
    ExecutionResource {
        denial:
            crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial,
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
    PreparedRootBudgetExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    IndexMaintenanceBudgetExceeded,
    IndexGenerationIdentityExhausted,
    ProviderIdentityMismatch,
    ProviderGenerationMismatch,
    SessionProtocolUnsupported,
    ProviderRejected,
    ProviderPanicked,
    TokenNotMintedForPlan,
    EmptyPhysicalSessionIdentity,
    SessionIdentityExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderSessionFailure {
    kind: WorthQueryProviderSessionDenialKind,
    stage: WorthQueryProviderSessionProtocolStage,
    recovery_posture: WorthQueryProviderSessionRecoveryPosture,
    detail: String,
    counters: WorthQueryProviderSessionProtocolCounters,
}

impl WorthQueryProviderSessionFailure {
    pub fn new(
        kind: WorthQueryProviderSessionDenialKind,
        stage: WorthQueryProviderSessionProtocolStage,
        detail: impl Into<String>,
        counters: WorthQueryProviderSessionProtocolCounters,
    ) -> Self {
        Self {
            kind,
            stage,
            recovery_posture: WorthQueryProviderSessionRecoveryPosture::Closed,
            detail: detail.into(),
            counters,
        }
    }

    pub(crate) fn unsupported() -> Self {
        Self::new(
            WorthQueryProviderSessionDenialKind::SessionProtocolUnsupported,
            WorthQueryProviderSessionProtocolStage::PlanReadmission,
            "installed provider does not implement the sealed session protocol",
            WorthQueryProviderSessionProtocolCounters::default(),
        )
    }

    pub fn kind(&self) -> WorthQueryProviderSessionDenialKind {
        self.kind
    }

    pub fn stage(&self) -> WorthQueryProviderSessionProtocolStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn recovery_posture(&self) -> WorthQueryProviderSessionRecoveryPosture {
        self.recovery_posture
    }

    pub fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }

    pub(super) fn at_stage(
        mut self,
        stage: WorthQueryProviderSessionProtocolStage,
        counters: WorthQueryProviderSessionProtocolCounters,
    ) -> Self {
        self.stage = stage;
        self.counters = counters;
        self
    }

    pub(in crate::domain_computation) fn with_recovery_posture(
        mut self,
        posture: WorthQueryProviderSessionRecoveryPosture,
    ) -> Self {
        self.recovery_posture = posture;
        self
    }
}
