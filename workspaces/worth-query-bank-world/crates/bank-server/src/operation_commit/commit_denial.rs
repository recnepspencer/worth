//! Bank-owned description of a Query commit denial.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankCommitDenialKind {
    ProviderRejected,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankCommitDenialStage {
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

pub(crate) const fn denial_kind(
    kind: WorthQueryApplicationCommitDenialKind,
) -> BankCommitDenialKind {
    use WorthQueryApplicationCommitDenialKind as Query;
    match kind {
        Query::ProviderRejected => BankCommitDenialKind::ProviderRejected,
        Query::ProductBasisStale => BankCommitDenialKind::ProductBasisStale,
        Query::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => BankCommitDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Query::RetentionCapacityExhausted => BankCommitDenialKind::RetentionCapacityExhausted,
        Query::RetentionIdentityExhausted => BankCommitDenialKind::RetentionIdentityExhausted,
        Query::SnapshotIdentityExhausted => BankCommitDenialKind::SnapshotIdentityExhausted,
        Query::CandidateIdentityExhausted => BankCommitDenialKind::CandidateIdentityExhausted,
        Query::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        } => BankCommitDenialKind::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        },
        Query::IdempotencyIntentDrift => BankCommitDenialKind::IdempotencyIntentDrift,
        Query::ElevationTransitionRequired => BankCommitDenialKind::ElevationTransitionRequired,
        Query::ElevationRequestProgramMismatch => {
            BankCommitDenialKind::ElevationRequestProgramMismatch
        }
        Query::ElevationApprovalProgramMismatch => {
            BankCommitDenialKind::ElevationApprovalProgramMismatch
        }
        Query::ElevationCloseProgramMismatch => BankCommitDenialKind::ElevationCloseProgramMismatch,
        Query::MandatoryReviewProgramMismatch => {
            BankCommitDenialKind::MandatoryReviewProgramMismatch
        }
        Query::DelegationActivationRequired => BankCommitDenialKind::DelegationActivationRequired,
        Query::CapabilityRevocationRequired => BankCommitDenialKind::CapabilityRevocationRequired,
    }
}

pub(crate) const fn denial_stage(
    stage: WorthQueryApplicationCommitDenialStage,
) -> BankCommitDenialStage {
    use WorthQueryApplicationCommitDenialStage as Query;
    match stage {
        Query::ProposalBinding => BankCommitDenialStage::ProposalBinding,
        Query::BridgePlanning => BankCommitDenialStage::BridgePlanning,
        Query::BasisAdmission => BankCommitDenialStage::BasisAdmission,
        Query::ResourceAdmission => BankCommitDenialStage::ResourceAdmission,
        Query::ManagedRunAdmission => BankCommitDenialStage::ManagedRunAdmission,
        Query::ProviderPlan => BankCommitDenialStage::ProviderPlan,
        Query::Idempotency => BankCommitDenialStage::Idempotency,
        Query::DecisionReadSet => BankCommitDenialStage::DecisionReadSet,
        Query::EffectLowering => BankCommitDenialStage::EffectLowering,
        Query::ElevationTransition => BankCommitDenialStage::ElevationTransition,
        Query::DelegationTransition => BankCommitDenialStage::DelegationTransition,
        Query::ProvisionalState => BankCommitDenialStage::ProvisionalState,
        Query::InvariantExecution => BankCommitDenialStage::InvariantExecution,
        Query::ProviderCommit => BankCommitDenialStage::ProviderCommit,
    }
}
