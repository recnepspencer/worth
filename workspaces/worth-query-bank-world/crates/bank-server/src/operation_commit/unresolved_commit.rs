//! Closed Bank description plus private Query recovery evidence.

use worth_query_host::facade::{
    installed::provider_session::{
        WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionProtocolStage,
    },
    primary_graph::{
        WorthQueryApplicationCommitRecoveryKind, WorthQueryApplicationUnresolvedCommitEvidence,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankCommitRecoveryKind {
    CommitRecoveryRequired,
    AbortRecoveryRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankProviderFailureKind {
    ForeignOperationAttempt,
    ForeignExecutionBasis,
    ForeignGraphAuthority,
    UndeclaredOperationScope,
    ResourceEnvelopeMismatch,
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
    ProviderIdentityMismatch,
    ProviderGenerationMismatch,
    SessionProtocolUnsupported,
    ProviderRejected,
    ProviderPanicked,
    TokenNotMintedForPlan,
    EmptyPhysicalSessionIdentity,
    SessionIdentityExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankProviderFailureStage {
    PlanAdmission,
    PlanReadmission,
    SessionPreparation,
    StagedPreparation,
    Commit,
    Abort,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BankUnresolvedCommitDescription {
    recovery: BankCommitRecoveryKind,
    provider_failure: BankProviderFailureKind,
    provider_stage: BankProviderFailureStage,
}

#[derive(Clone)]
pub struct BankUnresolvedCommitEvidence {
    description: BankUnresolvedCommitDescription,
}

impl BankUnresolvedCommitEvidence {
    pub(super) fn from_execution(execution: WorthQueryApplicationUnresolvedCommitEvidence) -> Self {
        let description = BankUnresolvedCommitDescription {
            recovery: recovery_kind(execution.recovery()),
            provider_failure: provider_failure_kind(execution.denial_kind()),
            provider_stage: provider_failure_stage(execution.stage()),
        };
        Self { description }
    }

    pub const fn recovery_kind(&self) -> BankCommitRecoveryKind {
        self.description.recovery
    }

    pub const fn provider_failure_kind(&self) -> BankProviderFailureKind {
        self.description.provider_failure
    }

    pub const fn provider_stage(&self) -> BankProviderFailureStage {
        self.description.provider_stage
    }
}

impl std::fmt::Debug for BankUnresolvedCommitEvidence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BankUnresolvedCommitEvidence")
            .field("description", &self.description)
            .finish()
    }
}

impl PartialEq for BankUnresolvedCommitEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
    }
}

impl Eq for BankUnresolvedCommitEvidence {}

const fn recovery_kind(kind: WorthQueryApplicationCommitRecoveryKind) -> BankCommitRecoveryKind {
    match kind {
        WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired => {
            BankCommitRecoveryKind::CommitRecoveryRequired
        }
        WorthQueryApplicationCommitRecoveryKind::AbortRecoveryRequired => {
            BankCommitRecoveryKind::AbortRecoveryRequired
        }
    }
}

const fn provider_failure_kind(
    kind: WorthQueryProviderSessionDenialKind,
) -> BankProviderFailureKind {
    use WorthQueryProviderSessionDenialKind as Query;
    match kind {
        Query::ForeignOperationAttempt => BankProviderFailureKind::ForeignOperationAttempt,
        Query::ForeignExecutionBasis => BankProviderFailureKind::ForeignExecutionBasis,
        Query::ForeignGraphAuthority => BankProviderFailureKind::ForeignGraphAuthority,
        Query::UndeclaredOperationScope => BankProviderFailureKind::UndeclaredOperationScope,
        Query::ResourceEnvelopeMismatch => BankProviderFailureKind::ResourceEnvelopeMismatch,
        Query::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => BankProviderFailureKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Query::RetentionCapacityExhausted => BankProviderFailureKind::RetentionCapacityExhausted,
        Query::RetentionIdentityExhausted => BankProviderFailureKind::RetentionIdentityExhausted,
        Query::SnapshotIdentityExhausted => BankProviderFailureKind::SnapshotIdentityExhausted,
        Query::CandidateIdentityExhausted => BankProviderFailureKind::CandidateIdentityExhausted,
        Query::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        } => BankProviderFailureKind::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        },
        Query::ProviderIdentityMismatch => BankProviderFailureKind::ProviderIdentityMismatch,
        Query::ProviderGenerationMismatch => BankProviderFailureKind::ProviderGenerationMismatch,
        Query::SessionProtocolUnsupported => BankProviderFailureKind::SessionProtocolUnsupported,
        Query::ProviderRejected => BankProviderFailureKind::ProviderRejected,
        Query::ProviderPanicked => BankProviderFailureKind::ProviderPanicked,
        Query::TokenNotMintedForPlan => BankProviderFailureKind::TokenNotMintedForPlan,
        Query::EmptyPhysicalSessionIdentity => {
            BankProviderFailureKind::EmptyPhysicalSessionIdentity
        }
        Query::SessionIdentityExhausted => BankProviderFailureKind::SessionIdentityExhausted,
    }
}

const fn provider_failure_stage(
    stage: WorthQueryProviderSessionProtocolStage,
) -> BankProviderFailureStage {
    use WorthQueryProviderSessionProtocolStage as Query;
    match stage {
        Query::PlanAdmission => BankProviderFailureStage::PlanAdmission,
        Query::PlanReadmission => BankProviderFailureStage::PlanReadmission,
        Query::SessionPreparation => BankProviderFailureStage::SessionPreparation,
        Query::StagedPreparation => BankProviderFailureStage::StagedPreparation,
        Query::Commit => BankProviderFailureStage::Commit,
        Query::Abort => BankProviderFailureStage::Abort,
    }
}
