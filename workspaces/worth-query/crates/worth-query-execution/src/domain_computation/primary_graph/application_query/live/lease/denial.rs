use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::{
    application_query::ApplicationQueryLiveCauseBinding, application_schema::ApplicationSchema,
};
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;

use super::super::outcome::{
    WorthQueryApplicationLiveCauseDenialKind, WorthQueryApplicationLiveOutcome,
};
use super::WorthQueryApplicationLiveLease;
use crate::domain_computation::primary_graph::{
    application_query::{
        authorized_read::WorthQueryAuthorizedApplicationReadDenial,
        read_execution::WorthQueryApplicationReadExecutionDenialKind,
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenialKind,
        WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    },
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

impl<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
    WorthQueryApplicationLiveLease<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    pub(super) fn handle_projection_denial(
        &mut self,
        kind: WorthQueryApplicationProjectionDenialKind,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        if matches!(
            kind,
            WorthQueryApplicationProjectionDenialKind::ValueDecode(_)
        ) {
            // A refused native decode cancels this read's custody. It cannot
            // consume the queued cause as though delivery had succeeded.
            self.terminate_stopped_read();
            return WorthQueryApplicationLiveOutcome::ProjectionDenied(kind);
        }
        if self.acknowledge_front() {
            WorthQueryApplicationLiveOutcome::ProjectionDenied(kind)
        } else {
            WorthQueryApplicationLiveOutcome::Unavailable
        }
    }

    pub(super) fn handle_admission_denial(
        &mut self,
        denial: WorthQueryApplicationQueryAdmissionDenial,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        match denial.kind() {
            WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(kind) => {
                self.execution_denied_outcome(kind)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::Cancelled => self.cancelled_outcome(),
            WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded => {
                self.deadline_exceeded_outcome()
            }
            WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_) => {
                let authorization = denial.into_authorization_denial().unwrap_or_else(|| {
                    WorthQueryOperationAuthorizationDenial::inconsistent(self.query.name())
                });
                self.authorization_denied_outcome(authorization)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal
            | WorthQueryApplicationQueryAdmissionDenialKind::ForeignPrincipal => {
                self.acknowledge_and_terminate(WorthQueryApplicationLiveOutcome::StalePrincipal)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::StaleScope
            | WorthQueryApplicationQueryAdmissionDenialKind::ForeignScope
            | WorthQueryApplicationQueryAdmissionDenialKind::ScopeTypeMismatch => {
                self.acknowledge_and_terminate(WorthQueryApplicationLiveOutcome::StaleScope)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
            | WorthQueryApplicationQueryAdmissionDenialKind::DisclosureAuthorizationMismatch
            | WorthQueryApplicationQueryAdmissionDenialKind::InternalComputationDenied => {
                let denial = WorthQueryOperationAuthorizationDenial::new(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::InconsistentDecision,
                    ),
                    self.query.name(),
                );
                self.authorization_denied_outcome(denial)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(_)
            | WorthQueryApplicationQueryAdmissionDenialKind::BasisUnsupported
            | WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis
            | WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis
            | WorthQueryApplicationQueryAdmissionDenialKind::WrongProviderBasis
            | WorthQueryApplicationQueryAdmissionDenialKind::ExpiredBasis
            | WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable
            | WorthQueryApplicationQueryAdmissionDenialKind::BranchMaterializationSuspended
            | WorthQueryApplicationQueryAdmissionDenialKind::ForeignHistoricalReceipt
            | WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable
            | WorthQueryApplicationQueryAdmissionDenialKind::SnapshotIdentityExhausted
            | WorthQueryApplicationQueryAdmissionDenialKind::RetentionCapacityExhausted
            | WorthQueryApplicationQueryAdmissionDenialKind::RetentionIdentityExhausted
            | WorthQueryApplicationQueryAdmissionDenialKind::ForeignContinuation
            | WorthQueryApplicationQueryAdmissionDenialKind::StaleContinuation
            | WorthQueryApplicationQueryAdmissionDenialKind::ContinuationParameterMismatch
            | WorthQueryApplicationQueryAdmissionDenialKind::ContinuationScopeMismatch
            | WorthQueryApplicationQueryAdmissionDenialKind::ContinuationProviderMismatch
            | WorthQueryApplicationQueryAdmissionDenialKind::ContinuationPageWidthUnsupported
            | WorthQueryApplicationQueryAdmissionDenialKind::LaneUnsupported
            | WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
            | WorthQueryApplicationQueryAdmissionDenialKind::Parameter(_)
            | WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded
            | WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted
            | WorthQueryApplicationQueryAdmissionDenialKind::CanonicalWorkDenied
            | WorthQueryApplicationQueryAdmissionDenialKind::GraphReadPlan(_)
            | WorthQueryApplicationQueryAdmissionDenialKind::GraphWorkAdmissionUnavailable
            | WorthQueryApplicationQueryAdmissionDenialKind::ActiveSnapshotCapacityExhausted { .. }
            | WorthQueryApplicationQueryAdmissionDenialKind::ExecutionShapeUnsupported => WorthQueryApplicationLiveOutcome::Unavailable,
        }
    }

    pub(super) fn handle_read_denial(
        &mut self,
        denial: WorthQueryAuthorizedApplicationReadDenial,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        match denial {
            WorthQueryAuthorizedApplicationReadDenial::StalePrincipal => {
                self.acknowledge_and_terminate(WorthQueryApplicationLiveOutcome::StalePrincipal)
            }
            WorthQueryAuthorizedApplicationReadDenial::StaleScope
            | WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope => {
                self.acknowledge_and_terminate(WorthQueryApplicationLiveOutcome::StaleScope)
            }
            WorthQueryAuthorizedApplicationReadDenial::Authorization(authorization) => {
                self.authorization_denied_outcome(authorization)
            }
            WorthQueryAuthorizedApplicationReadDenial::Read(read) => match read.kind() {
                WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(kind) => {
                    self.execution_denied_outcome(kind)
                }
                WorthQueryApplicationReadExecutionDenialKind::Cancelled => self.cancelled_outcome(),
                WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded => {
                    self.deadline_exceeded_outcome()
                }
                WorthQueryApplicationReadExecutionDenialKind::TargetIdentityNotFound
                | WorthQueryApplicationReadExecutionDenialKind::TargetIdentityLookupOverflow => {
                    self.acknowledge_cause_denial(
                        WorthQueryApplicationLiveCauseDenialKind::TargetIdentityUnavailable,
                    )
                }
                WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable => self
                    .acknowledge_cause_denial(
                        WorthQueryApplicationLiveCauseDenialKind::TargetOutsideScope,
                    ),
                WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch
                | WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable => self
                    .acknowledge_cause_denial(
                        WorthQueryApplicationLiveCauseDenialKind::ResultShapeUnavailable,
                    ),
                WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::ResultLimitExceeded
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationBoundaryRejected
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationGenerationChanged
                | WorthQueryApplicationReadExecutionDenialKind::ContinuationPageWidthInvalid
                | WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded
                | WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded => {
                    WorthQueryApplicationLiveOutcome::Unavailable
                }
            },
            WorthQueryAuthorizedApplicationReadDenial::Session => {
                WorthQueryApplicationLiveOutcome::Unavailable
            }
        }
    }

    fn authorization_denied_outcome(
        &mut self,
        denial: WorthQueryOperationAuthorizationDenial,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        match denial.kind() {
            WorthQueryOperationAuthorizationDenialKind::Stopped(Ok(kind)) => {
                self.execution_denied_outcome(kind)
            }
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
            )) => self.cancelled_outcome(),
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
            )) => self.deadline_exceeded_outcome(),
            WorthQueryOperationAuthorizationDenialKind::Refused(_) => self
                .acknowledge_and_terminate(WorthQueryApplicationLiveOutcome::AuthorizationDenied(
                    Box::new(denial),
                )),
        }
    }

    /// Ends this lease with the read's own request cause, retaining any failed termination.
    pub fn end_with_read_stop(
        &mut self,
        cause: Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        match cause {
            Ok(kind) => self.execution_denied_outcome(kind),
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled) => {
                self.cancelled_outcome()
            }
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut) => {
                self.deadline_exceeded_outcome()
            }
        }
    }

    pub(super) fn execution_denied_outcome(
        &mut self,
        kind: crate::domain_computation::WorthQueryProviderSessionDenialKind,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        self.terminate_stopped_read();
        WorthQueryApplicationLiveOutcome::ExecutionRequest(kind)
    }

    pub(super) fn cancelled_outcome(
        &mut self,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        self.terminate_stopped_read();
        WorthQueryApplicationLiveOutcome::Cancelled
    }

    pub(super) fn deadline_exceeded_outcome(
        &mut self,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        self.terminate_stopped_read();
        WorthQueryApplicationLiveOutcome::DeadlineExceeded
    }

    pub(super) fn terminate_stopped_read(&mut self) {
        if !self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
            self.termination_failed = true;
        }
    }

    /// Whether ending this lease after a stopped read also failed.
    pub fn termination_failed(&self) -> bool {
        self.termination_failed
    }
}
