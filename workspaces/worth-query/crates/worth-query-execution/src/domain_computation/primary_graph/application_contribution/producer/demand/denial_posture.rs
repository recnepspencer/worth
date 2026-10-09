//! Retry meaning of typed output-demand stops.

use super::{WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandRecoveryPosture};

impl WorthQueryOutputDemandDenialKind {
    pub(super) const fn default_recovery_posture(self) -> WorthQueryOutputDemandRecoveryPosture {
        use WorthQueryOutputDemandRecoveryPosture::{Retryable, Terminal};
        match self {
            Self::ExecutionRequest(cause) => {
                if cause.is_transient() {
                    Retryable
                } else {
                    Terminal
                }
            }
            Self::ProductSelection(denial) => {
                if denial.is_transient() {
                    Retryable
                } else {
                    Terminal
                }
            }
            Self::SchedulingDeferred => Retryable,
            Self::SourcePrincipal(_)
            | Self::SourceScope(_)
            | Self::SourceQueryInstallation(_)
            | Self::SourceQueryAdmission(_)
            | Self::SourceQueryExecution(_) => Terminal,
            Self::ForeignSource
            | Self::MissingApplicableProducer
            | Self::AmbiguousApplicableProducer
            | Self::ProducerUnavailable
            | Self::RequestAuthorization(_)
            | Self::SchedulingRejected
            | Self::PublicationStale
            | Self::NoEffect
            | Self::Superseded
            | Self::Cancelled
            | Self::TimedOut
            | Self::WorkBudgetExceeded
            | Self::RetentionBudgetExceeded
            | Self::PublicationCapacityExceeded
            | Self::ForeignDemand
            | Self::ForeignSettlement
            | Self::IncompleteDependencyCoverage
            | Self::RetainedBasisUnavailable
            | Self::Closed
            | Self::DuplicatePerformedSource => Terminal,
        }
    }
}

impl WorthQueryOutputDemandDenialKind {
    /// The request was refused authorization: its interruption, the security
    /// basis it could not select on the branch, or what it may do there. A
    /// basis the branch has no room to observe now is a retryable product
    /// selection stop, as it is for the demand's own selection.
    pub(in crate::domain_computation::primary_graph) const fn of_request_authorization(
        kind: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
    ) -> Self {
        use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Kind;
        match kind {
            Kind::Cancelled => Self::Cancelled,
            Kind::DeadlineExceeded => Self::TimedOut,
            Kind::ProductSecurityBasis(admission) => Self::ProductSelection(admission),
            other => Self::RequestAuthorization(other),
        }
    }

    /// The request's principal did not resolve: the request's interruption or
    /// budgets, or the principal it names. Never the producer's own stop.
    pub(in crate::domain_computation::primary_graph) const fn of_principal_resolution(
        kind: crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind,
    ) -> Self {
        use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind as Kind;
        match kind {
            Kind::Cancelled => Self::Cancelled,
            Kind::DeadlineExceeded => Self::TimedOut,
            Kind::ProjectionWorkBudgetExceeded => Self::WorkBudgetExceeded,
            Kind::ProjectionPreparationMemoryExhausted => Self::RetentionBudgetExceeded,
            other => Self::SourcePrincipal(other),
        }
    }

    /// The request's scope did not resolve, as [`Self::of_principal_resolution`].
    pub(in crate::domain_computation::primary_graph) const fn of_scope_resolution(
        kind: crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind,
    ) -> Self {
        use crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind as Kind;
        match kind {
            Kind::Cancelled => Self::Cancelled,
            Kind::DeadlineExceeded => Self::TimedOut,
            Kind::ProjectionWorkBudgetExceeded => Self::WorkBudgetExceeded,
            Kind::ProjectionPreparationMemoryExhausted => Self::RetentionBudgetExceeded,
            other => Self::SourceScope(other),
        }
    }

    /// Execution observed an interruption after the advancement opened.
    pub(super) const fn of_execution_interruption(
        interruption: crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption,
    ) -> Self {
        use crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption as Interruption;
        match interruption {
            Interruption::Cancelled => Self::Cancelled,
            Interruption::DeadlineExceeded => Self::TimedOut,
        }
    }

    /// Delivery is in flight; its interruption keeps the demand owner's kind.
    pub(super) const fn of_execution_stop(
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    ) -> Self {
        use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial as Denial;
        match cause {
            Denial::Interrupted(interruption) => Self::of_execution_interruption(interruption),
            Denial::Resource(_)
            | Denial::NestedOpening
            | Denial::ForeignPhase
            | Denial::NestedStopped
            | Denial::Panicked => Self::ExecutionRequest(cause),
        }
    }

    /// The request's interruption, met while verifying what an output
    /// consumed.
    pub(in crate::domain_computation::primary_graph) const fn of_interruption(
        interruption: worth_relational::facade::mvcc::RelationalOperationInterruption,
    ) -> Self {
        use worth_relational::facade::mvcc::RelationalOperationInterruption as Interruption;
        match interruption {
            Interruption::Cancelled => Self::Cancelled,
            Interruption::TimedOut => Self::TimedOut,
        }
    }
}
