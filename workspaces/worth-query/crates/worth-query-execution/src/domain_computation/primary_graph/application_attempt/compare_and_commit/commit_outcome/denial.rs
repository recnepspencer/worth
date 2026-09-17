//! Pre-publication application denial categories and owner evidence.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitDenialKind {
    ProviderRejected,
    CustomInvariantDenied,
    CandidateValidatorWorkExceeded {
        maximum_work: usize,
        required_work: usize,
    },
    ProductBasisStale,
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
    ApplicationProgramRequired,
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
    detail: Option<std::sync::Arc<str>>,
    custom_invariant: Option<crate::domain_computation::WorthQueryCustomInvariantDenial>,
}

impl WorthQueryApplicationCommitDenial {
    pub const fn kind(&self) -> WorthQueryApplicationCommitDenialKind {
        self.kind
    }

    pub const fn stage(&self) -> WorthQueryApplicationCommitDenialStage {
        self.stage
    }

    /// Returns provider-owned diagnostic detail when the rejection boundary
    /// supplies a concrete cause.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn custom_invariant_denial(
        &self,
    ) -> Option<&crate::domain_computation::WorthQueryCustomInvariantDenial> {
        self.custom_invariant.as_ref()
    }

    pub fn custom_invariant_violation_identity(
        &self,
    ) -> Option<&worth_relational::facade::transactions::CustomInvariantSemanticIdentity> {
        match self.custom_invariant.as_ref()? {
            crate::domain_computation::WorthQueryCustomInvariantDenial::Violation { identity } => {
                Some(identity)
            }
            crate::domain_computation::WorthQueryCustomInvariantDenial::Failure { .. } => None,
        }
    }

    pub fn custom_invariant_failure(
        &self,
    ) -> Option<(
        &worth_relational::facade::transactions::CustomInvariantFailureIdentity,
        worth_relational::facade::transactions::CustomInvariantFailurePhase,
        worth_relational::facade::transactions::ResultCustomInvariantFailureKind,
    )> {
        match self.custom_invariant.as_ref()? {
            crate::domain_computation::WorthQueryCustomInvariantDenial::Failure {
                identity,
                phase,
                failure,
            } => Some((identity, *phase, *failure)),
            crate::domain_computation::WorthQueryCustomInvariantDenial::Violation { .. } => None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn custom_invariant_denied(
        stage: WorthQueryApplicationCommitDenialStage,
        custom_invariant: crate::domain_computation::WorthQueryCustomInvariantDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CustomInvariantDenied,
            stage,
            detail: None,
            custom_invariant: Some(custom_invariant),
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn provider_rejected(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn provider_rejected_with_detail(
        stage: WorthQueryApplicationCommitDenialStage,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage,
            detail: Some(detail.into()),
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn candidate_validator_work_exceeded(
        stage: WorthQueryApplicationCommitDenialStage,
        maximum_work: usize,
        required_work: usize,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CandidateValidatorWorkExceeded {
                maximum_work,
                required_work,
            },
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn product_basis_stale(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProductBasisStale,
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn active_snapshot_capacity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
        maximum_active_snapshots: usize,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn retention_capacity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::RetentionCapacityExhausted,
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn snapshot_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::SnapshotIdentityExhausted,
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn retention_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::RetentionIdentityExhausted,
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn candidate_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CandidateIdentityExhausted,
            stage,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn prepared_root_budget_exhausted(
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
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency_intent_drift(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_transition_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationTransitionRequired,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn delegation_activation_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::DelegationActivationRequired,
            stage: WorthQueryApplicationCommitDenialStage::DelegationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn capability_revocation_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CapabilityRevocationRequired,
            stage: WorthQueryApplicationCommitDenialStage::DelegationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation) const fn application_program_required() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_request_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationRequestProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_approval_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationApprovalProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_close_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationCloseProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn mandatory_review_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::MandatoryReviewProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            custom_invariant: None,
        }
    }
}
