//! Compare-and-commit outcome and denial taxonomy.

use super::{super::WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationCommitDeferred};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationStaleAttempt {
    stale_fact_count: usize,
}

impl WorthQueryApplicationStaleAttempt {
    pub const fn stale_fact_count(self) -> usize {
        self.stale_fact_count
    }

    pub(in super::super) const fn new(stale_fact_count: usize) -> Self {
        Self { stale_fact_count }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitDenialKind {
    ProviderRejected,
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
    IdempotencyIntentDrift,
    ElevationTransitionRequired,
    ElevationRequestProgramMismatch,
    ElevationApprovalProgramMismatch,
    ElevationCloseProgramMismatch,
    MandatoryReviewProgramMismatch,
    DelegationActivationRequired,
    CapabilityRevocationRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitDenialStage {
    ProposalBinding,
    BridgePlanning,
    BasisAdmission,
    ResourceAdmission,
    ManagedRunAdmission,
    ProviderPlan,
    Idempotency,
    DecisionReadSet,
    EffectLowering,
    ElevationTransition,
    DelegationTransition,
    ProvisionalState,
    InvariantExecution,
    ProviderCommit,
}

#[derive(Debug)]
pub struct WorthQueryApplicationCommitDenial {
    kind: WorthQueryApplicationCommitDenialKind,
    stage: WorthQueryApplicationCommitDenialStage,
}

impl WorthQueryApplicationCommitDenial {
    pub const fn kind(&self) -> WorthQueryApplicationCommitDenialKind {
        self.kind
    }

    pub const fn stage(&self) -> WorthQueryApplicationCommitDenialStage {
        self.stage
    }

    pub(in super::super) const fn provider_rejected(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage,
        }
    }

    pub(in super::super) const fn active_snapshot_capacity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
        maximum_active_snapshots: usize,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            stage,
        }
    }

    pub(in super::super) const fn retention_capacity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::RetentionCapacityExhausted,
            stage,
        }
    }

    pub(in super::super) const fn snapshot_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::SnapshotIdentityExhausted,
            stage,
        }
    }

    pub(in super::super) const fn retention_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::RetentionIdentityExhausted,
            stage,
        }
    }

    pub(in super::super) const fn candidate_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CandidateIdentityExhausted,
            stage,
        }
    }

    pub(in super::super) const fn prepared_root_budget_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
        maximum_bytes: u64,
        required_bytes: u64,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::PreparedRootBudgetExhausted {
                maximum_bytes,
                required_bytes,
            },
            stage,
        }
    }

    pub(in super::super) const fn idempotency_intent_drift() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
        }
    }

    pub(in super::super) const fn elevation_transition_required() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationTransitionRequired,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
        }
    }

    pub(in super::super) const fn delegation_activation_required() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::DelegationActivationRequired,
            stage: WorthQueryApplicationCommitDenialStage::DelegationTransition,
        }
    }

    pub(in super::super) const fn capability_revocation_required() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CapabilityRevocationRequired,
            stage: WorthQueryApplicationCommitDenialStage::DelegationTransition,
        }
    }

    pub(in super::super) const fn elevation_request_program_mismatch() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationRequestProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
        }
    }

    pub(in super::super) const fn elevation_approval_program_mismatch() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationApprovalProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
        }
    }

    pub(in super::super) const fn elevation_close_program_mismatch() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationCloseProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
        }
    }

    pub(in super::super) const fn mandatory_review_program_mismatch() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::MandatoryReviewProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
        }
    }
}

#[derive(Debug)]
pub enum WorthQueryApplicationCommitOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    Committed(super::WorthQueryApplicationCommitReceipt),
    AlreadyCommitted(super::WorthQueryApplicationCommitReceipt),
    Stale(WorthQueryApplicationStaleAttempt),
    Cancelled,
    TimedOut,
    Denied(WorthQueryApplicationCommitDenial),
    Aborted,
    Deferred(WorthQueryApplicationCommitDeferred),
    SettlementDeferred(WorthQueryApplicationSettlementDeferred),
    Indeterminate(WorthQueryApplicationUnresolvedCommitEvidence),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationSettlementNextAction {
    RecoverDeferredApplicationSettlement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationSettlementDeferred {
    stage: crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage,
    detail: String,
    counters:
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters,
    settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    publication_failure_stage:
        Option<crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage>,
    publication_failure_detail: Option<String>,
    publication_failure_counters: Option<
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters,
    >,
    idempotency_binding: WorthQueryApplicationIdempotencyBinding,
    branch: worth_relational::facade::history::BranchId,
}

impl WorthQueryApplicationSettlementDeferred {
    pub(in crate::domain_computation::primary_graph) fn from_provider_session(
        deferred: crate::domain_computation::provider_session::WorthQueryProviderSessionSettlementDeferred,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
        branch: worth_relational::facade::history::BranchId,
    ) -> Self {
        let publication_failure = deferred.publication_failure();
        Self {
            stage: deferred.stage(),
            detail: deferred.detail().to_owned(),
            counters: deferred.counters(),
            settlement: deferred.settlement().clone(),
            publication_failure_stage: publication_failure.map(|failure| failure.stage()),
            publication_failure_detail: publication_failure
                .map(|failure| failure.detail().to_owned()),
            publication_failure_counters: publication_failure.map(|failure| failure.counters()),
            idempotency_binding,
            branch,
        }
    }

    pub const fn stage(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters
    {
        self.counters
    }

    pub(in crate::domain_computation::primary_graph) fn settlement(
        &self,
    ) -> &worth_relational::facade::publication::DeferredPublicationSettlement {
        &self.settlement
    }

    pub const fn publication_failure_stage(
        &self,
    ) -> Option<crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage>
    {
        self.publication_failure_stage
    }

    pub fn publication_failure_detail(&self) -> Option<&str> {
        self.publication_failure_detail.as_deref()
    }

    pub const fn publication_failure_counters(
        &self,
    ) -> Option<
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters,
    > {
        self.publication_failure_counters
    }

    pub const fn next_action(&self) -> WorthQueryApplicationSettlementNextAction {
        WorthQueryApplicationSettlementNextAction::RecoverDeferredApplicationSettlement
    }

    pub fn commit_id(&self) -> worth_relational::facade::history::CommitId {
        self.settlement.commit().commit_id
    }

    pub(in crate::domain_computation::primary_graph) const fn requires_idempotency_readmission(
        &self,
    ) -> bool {
        self.publication_failure_detail.is_some()
    }

    pub(in crate::domain_computation::primary_graph) const fn idempotency_binding(
        &self,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency_binding
    }

    pub(in crate::domain_computation::primary_graph) fn branch(
        &self,
    ) -> &worth_relational::facade::history::BranchId {
        &self.branch
    }
}

/// Correlation evidence retained when commit outcome is unresolved (R8.26 / C3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationUnresolvedCommitEvidence {
    recovery: WorthQueryApplicationCommitRecoveryKind,
    denial_kind: crate::domain_computation::provider_session::WorthQueryProviderSessionDenialKind,
    stage: crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage,
    detail: String,
}

/// Distinguishes commit-path vs abort-path recovery requirement (R8.26).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitRecoveryKind {
    CommitRecoveryRequired,
    AbortRecoveryRequired,
}

impl WorthQueryApplicationUnresolvedCommitEvidence {
    pub(in crate::domain_computation::primary_graph) fn from_provider_session_failure(
        recovery: WorthQueryApplicationCommitRecoveryKind,
        failure: &crate::domain_computation::provider_session::WorthQueryProviderSessionFailure,
    ) -> Self {
        Self {
            recovery,
            denial_kind: failure.kind(),
            stage: failure.stage(),
            detail: failure.detail().to_owned(),
        }
    }

    pub const fn recovery(&self) -> WorthQueryApplicationCommitRecoveryKind {
        self.recovery
    }

    pub const fn denial_kind(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionDenialKind {
        self.denial_kind
    }

    pub const fn stage(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
