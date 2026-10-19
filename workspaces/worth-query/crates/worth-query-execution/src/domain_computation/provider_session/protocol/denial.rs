mod native_preparation;

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
    IndexMaintenanceBudgetExceeded,
    IndexGenerationIdentityExhausted,
    ProviderIdentityMismatch,
    ProviderGenerationMismatch,
    SessionProtocolUnsupported,
    AllocationDenied,
    ProviderRejected,
    ProviderPanicked,
    TokenNotMintedForPlan,
    EmptyPhysicalSessionIdentity,
    SessionIdentityExhausted,
}

/// A provider session's exact failure, protocol stage and recovery posture.
/// Native preparation and allocation causes remain available for inspection;
/// this description does not itself grant retry or cleanup authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderSessionFailure {
    kind: WorthQueryProviderSessionDenialKind,
    stage: WorthQueryProviderSessionProtocolStage,
    recovery_posture: WorthQueryProviderSessionRecoveryPosture,
    detail: String,
    counters: WorthQueryProviderSessionProtocolCounters,
    allocation: Option<worth_execution::ExecutionAllocationDenial>,
    native_preparation: Option<std::sync::Arc<native_preparation::NativePreparationFailure>>,
    output_verification_capacity:
        Option<crate::domain_computation::primary_graph::VerificationCapacityStop>,
    checked_read: Option<
        Box<
            Result<
                WorthQueryProviderSessionDenialKind,
                super::WorthQueryProviderSessionControlStopKind,
            >,
        >,
    >,
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
            allocation: None,
            native_preparation: None,
            checked_read: None,
            output_verification_capacity: None,
        }
    }

    pub(in crate::domain_computation) fn with_output_verification_capacity_stop(
        mut self,
        stop: crate::domain_computation::primary_graph::VerificationCapacityStop,
    ) -> Self {
        self.output_verification_capacity = Some(stop);
        self
    }

    /// Original capacity refusal from consumed-output verification before
    /// publication, separate from native transaction preparation. Diagnostic
    /// data does not change the failure's category or recovery posture.
    pub const fn output_verification_capacity_stop(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::VerificationCapacityStop> {
        self.output_verification_capacity
    }

    pub(in crate::domain_computation) fn checked_read_stopped(
        stop: Result<
            WorthQueryProviderSessionDenialKind,
            super::WorthQueryProviderSessionControlStopKind,
        >,
        stage: WorthQueryProviderSessionProtocolStage,
        detail: impl Into<String>,
    ) -> Self {
        let mut failure = Self::new(
            stop.unwrap_or(WorthQueryProviderSessionDenialKind::ProviderRejected),
            stage,
            detail,
            Default::default(),
        );
        failure.checked_read = Some(Box::new(stop));
        failure
    }

    /// Exact request stop retained while the protocol holds publication custody.
    pub fn checked_read_stop(
        &self,
    ) -> Option<
        Result<
            WorthQueryProviderSessionDenialKind,
            super::WorthQueryProviderSessionControlStopKind,
        >,
    > {
        self.checked_read.as_deref().copied()
    }

    pub(in crate::domain_computation) fn allocation_denied(
        denial: worth_execution::ExecutionAllocationDenial,
    ) -> Self {
        let mut failure = Self::new(
            WorthQueryProviderSessionDenialKind::AllocationDenied,
            WorthQueryProviderSessionProtocolStage::Commit,
            format!("completed touched-record backing allocation denied: {denial:?}"),
            WorthQueryProviderSessionProtocolCounters::default(),
        );
        failure.allocation = Some(denial);
        failure
    }

    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        self.allocation.as_ref()
    }

    pub(in crate::domain_computation) fn with_native_preparation_error(
        mut self,
        error: worth_relational::facade::mvcc::TransactionCommitError,
    ) -> Self {
        self.native_preparation = Some(std::sync::Arc::new(
            native_preparation::NativePreparationFailure(error),
        ));
        self
    }

    /// Exact native refusal at the pre-publication candidate preparation port.
    pub fn native_preparation_error(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::TransactionCommitError> {
        self.native_preparation.as_deref().map(|failure| &failure.0)
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
