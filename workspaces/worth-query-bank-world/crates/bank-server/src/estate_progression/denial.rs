//! Public Bank denial surface for estate progression.

mod idempotency;
mod lifecycle_projection;
mod operation_projection;

pub use idempotency::BankEstateIdempotencyResolutionDenial;
pub use lifecycle_projection::BankEstateLifecycleProjectionDenial;
pub use operation_projection::BankEstateOperationProjectionDenial;

use worth_query_host::facade::domain::{
    WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationOperationInstallationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryElevationApprovalAuthorizationDenial, WorthQueryElevationCloseAuthorizationDenial,
    WorthQueryInvariantDecisionPlanDenial, WorthQueryMandatoryReviewAuthorizationDenial,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationProjectionDenial,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};

use super::{
    BankCapabilityDelegationProjectionDenial, BankCapabilityRevocationProjectionDenial,
    BankDeathNotificationProjectionDenial, BankEstateCaseOpeningProjectionDenial,
    BankEstateDisbursementProjectionDenial, BankEstateFreezeProjectionDenial,
    BankEstateReleaseProjectionDenial, BankExecutorRecognitionProjectionDenial,
    BankInvariantDecisionPlanDenial,
};

#[derive(Debug)]
pub enum BankEstateProgressionDenial {
    ProductSelection(worth_query_host::facade::product::WorthQueryProductBranchAdmissionDenial),
    CapabilityInstallation(crate::BankApplicationCapabilityInstallationDenialKind),
    OperationInstallation(crate::BankOperationInstallationDenial),
    Authorization(crate::BankAuthorizationDenial),
    ApprovalAuthorization(crate::BankAuthorizationDenial),
    CloseAuthorization(crate::BankAuthorizationDenial),
    ReviewAuthorization(crate::BankAuthorizationDenial),
    CommandInput(&'static str),
    Projection(BankEstateOperationProjectionDenial),
    DecisionProjection(BankInvariantDecisionPlanDenial),
    FreezeProjection(BankEstateFreezeProjectionDenial),
    DeathNotificationProjection(BankDeathNotificationProjectionDenial),
    CaseOpeningProjection(BankEstateCaseOpeningProjectionDenial),
    ExecutorRecognitionProjection(BankExecutorRecognitionProjectionDenial),
    EstateReleaseProjection(BankEstateReleaseProjectionDenial),
    EstateDisbursementProjection(BankEstateDisbursementProjectionDenial),
    Proposal(bank_domain::proposals::BankProposalDenial),
    CommitPreparation(crate::operation_commit::BankCommitPreparationDenial),
    CapabilityDelegationProjection(BankCapabilityDelegationProjectionDenial),
    CapabilityRevocationProjection(BankCapabilityRevocationProjectionDenial),
    Idempotency(BankEstateIdempotencyResolutionDenial),
    LifecycleProjection(BankEstateLifecycleProjectionDenial),
    Attempt(crate::BankCommitPreparationDenial),
    Recovery(BankRecoveryDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankRecoveryDenialKind {
    RecoveryNotAdmitted,
    RecoveryAlreadyMinted,
    RuntimeMismatch,
    SchemaMismatch,
    BranchMismatch,
    ApplicationBindingGenerationMismatch,
    OperationMismatch,
    GovernedInputMismatch,
    AttemptMismatch,
    PrincipalScopeMismatch,
    IdempotencyMismatch,
    ForeignIdempotencyRead,
    ProviderPostureMismatch,
    CorrelationMismatch,
    CompatibilityGenerationMismatch,
    Expired,
    AlreadyTerminal,
    ForeignPrincipal,
    ForeignRuntime,
    ForeignBranchEqualOrdinal,
    TransitionNotAdmitted,
    CompensationNotAdmitted,
    ReconciliationNotAdmitted,
    FreshAuthorityDenied,
    DisclosureAdmissionRequired,
    CurrentPolicyDenied,
    UnresolvedExternalPosture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankRecoveryDenial {
    kind: BankRecoveryDenialKind,
}

impl BankRecoveryDenial {
    pub const fn kind(self) -> BankRecoveryDenialKind {
        self.kind
    }

    pub(crate) fn from_query(denial: WorthQueryRecoveryHandleDenial) -> Self {
        use BankRecoveryDenialKind as Bank;
        use WorthQueryRecoveryHandleDenialKind as Query;
        let kind = match denial.kind() {
            Query::RecoveryNotAdmitted => Bank::RecoveryNotAdmitted,
            Query::RecoveryAlreadyMinted => Bank::RecoveryAlreadyMinted,
            Query::RuntimeMismatch => Bank::RuntimeMismatch,
            Query::SchemaMismatch => Bank::SchemaMismatch,
            Query::BranchMismatch => Bank::BranchMismatch,
            Query::ApplicationBindingGenerationMismatch => {
                Bank::ApplicationBindingGenerationMismatch
            }
            Query::OperationMismatch => Bank::OperationMismatch,
            Query::GovernedInputMismatch => Bank::GovernedInputMismatch,
            Query::AttemptMismatch => Bank::AttemptMismatch,
            Query::PrincipalScopeMismatch => Bank::PrincipalScopeMismatch,
            Query::IdempotencyMismatch => Bank::IdempotencyMismatch,
            Query::ForeignIdempotencyRead => Bank::ForeignIdempotencyRead,
            Query::ProviderPostureMismatch => Bank::ProviderPostureMismatch,
            Query::CorrelationMismatch => Bank::CorrelationMismatch,
            Query::CompatibilityGenerationMismatch => Bank::CompatibilityGenerationMismatch,
            Query::Expired => Bank::Expired,
            Query::AlreadyTerminal => Bank::AlreadyTerminal,
            Query::ForeignPrincipal => Bank::ForeignPrincipal,
            Query::ForeignRuntime => Bank::ForeignRuntime,
            Query::ForeignBranchEqualOrdinal => Bank::ForeignBranchEqualOrdinal,
            Query::TransitionNotAdmitted => Bank::TransitionNotAdmitted,
            Query::CompensationNotAdmitted => Bank::CompensationNotAdmitted,
            Query::ReconciliationNotAdmitted => Bank::ReconciliationNotAdmitted,
            Query::FreshAuthorityDenied => Bank::FreshAuthorityDenied,
            Query::DisclosureAdmissionRequired => Bank::DisclosureAdmissionRequired,
            Query::CurrentPolicyDenied => Bank::CurrentPolicyDenied,
            Query::UnresolvedExternalPosture => Bank::UnresolvedExternalPosture,
        };
        Self { kind }
    }
}

impl std::fmt::Display for BankEstateProgressionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProductSelection(denial) => {
                write!(formatter, "product selection denied: {denial:?}")
            }
            Self::CapabilityInstallation(_) => {
                formatter.write_str("capability-installation-denied")
            }
            Self::OperationInstallation(denial) => formatter.write_str(denial.code()),
            Self::Authorization(denial)
            | Self::ApprovalAuthorization(denial)
            | Self::CloseAuthorization(denial)
            | Self::ReviewAuthorization(denial) => formatter.write_str(denial.code()),
            Self::CommandInput(operation) => {
                write!(formatter, "invalid estate lifecycle input for {operation}")
            }
            Self::Projection(_) => formatter.write_str("operation-projection-denied"),
            Self::DecisionProjection(_) => formatter.write_str("decision-projection-denied"),
            Self::FreezeProjection(denial) => denial.fmt(formatter),
            Self::DeathNotificationProjection(denial) => denial.fmt(formatter),
            Self::CaseOpeningProjection(denial) => denial.fmt(formatter),
            Self::ExecutorRecognitionProjection(denial) => denial.fmt(formatter),
            Self::EstateReleaseProjection(denial) => denial.fmt(formatter),
            Self::EstateDisbursementProjection(denial) => denial.fmt(formatter),
            Self::Proposal(denial) => denial.fmt(formatter),
            Self::CommitPreparation(denial) => denial.fmt(formatter),
            Self::CapabilityDelegationProjection(denial) => denial.fmt(formatter),
            Self::CapabilityRevocationProjection(denial) => denial.fmt(formatter),
            Self::Idempotency(_) => formatter.write_str("idempotency-resolution-denied"),
            Self::LifecycleProjection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
            Self::Recovery(denial) => {
                write!(formatter, "recovery handle denied: {:?}", denial.kind())
            }
        }
    }
}

impl BankEstateProgressionDenial {
    pub(crate) const fn from_product_selection(
        denial: worth_query_host::facade::product::WorthQueryProductBranchAdmissionDenial,
    ) -> Self {
        Self::ProductSelection(denial)
    }

    pub(crate) fn from_capability_installation(
        denial: WorthQueryApplicationCapabilityInstallationDenial,
    ) -> Self {
        Self::CapabilityInstallation(
            crate::BankApplicationCapabilityInstallationDenialKind::from_query(denial.kind()),
        )
    }

    pub(crate) fn from_operation_installation(
        denial: WorthQueryApplicationOperationInstallationDenial,
    ) -> Self {
        Self::OperationInstallation(crate::BankOperationInstallationDenial::from_query(
            denial.kind(),
        ))
    }

    pub(crate) fn from_authorization(denial: WorthQueryOperationAuthorizationDenial) -> Self {
        Self::Authorization(crate::BankAuthorizationDenial::from_query(denial))
    }

    pub(crate) fn from_approval_authorization_ref(
        denial: &WorthQueryElevationApprovalAuthorizationDenial,
    ) -> Self {
        Self::ApprovalAuthorization(crate::BankAuthorizationDenial::from_query(
            denial.denial().clone(),
        ))
    }

    pub(crate) fn from_close_authorization_ref(
        denial: &WorthQueryElevationCloseAuthorizationDenial,
    ) -> Self {
        Self::CloseAuthorization(crate::BankAuthorizationDenial::from_query(
            denial.denial().clone(),
        ))
    }

    pub(crate) fn from_review_authorization_ref(
        denial: &WorthQueryMandatoryReviewAuthorizationDenial,
    ) -> Self {
        Self::ReviewAuthorization(crate::BankAuthorizationDenial::from_query(
            denial.denial().clone(),
        ))
    }

    pub(crate) fn from_projection(denial: WorthQueryOperationProjectionDenial) -> Self {
        Self::Projection(operation_projection::from_query(denial))
    }

    pub(crate) fn from_decision_projection(denial: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionProjection(BankInvariantDecisionPlanDenial::from_query(denial.kind()))
    }

    pub(crate) fn from_idempotency(
        denial: WorthQueryApplicationIdempotencyResolutionDenial,
    ) -> Self {
        Self::Idempotency(idempotency::from_query(denial))
    }

    pub(crate) fn from_attempt(denial: WorthQueryApplicationAttemptDenial) -> Self {
        Self::Attempt(denial.into())
    }

    pub(crate) fn from_recovery(denial: WorthQueryRecoveryHandleDenial) -> Self {
        Self::Recovery(BankRecoveryDenial::from_query(denial))
    }
}

impl From<WorthQueryApplicationAttemptDenial> for BankEstateProgressionDenial {
    fn from(denial: WorthQueryApplicationAttemptDenial) -> Self {
        Self::Attempt(denial.into())
    }
}

impl std::error::Error for BankEstateProgressionDenial {}
