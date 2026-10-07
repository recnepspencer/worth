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
        WorthQueryApplicationProjection, WorthQueryApplicationQueryAdmissionDenial,
        WorthQueryApplicationQueryAdmissionDenialKind,
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
    pub(super) fn handle_admission_denial(
        &mut self,
        denial: WorthQueryApplicationQueryAdmissionDenial,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        match denial.kind() {
            WorthQueryApplicationQueryAdmissionDenialKind::Handle(handle) => {
                self.handle_denial_outcome(handle)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::Cancelled => self.cancelled_outcome(),
            WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded => {
                self.deadline_exceeded_outcome()
            }
            WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                WorthQueryOperationAuthorizationDenialKind::Handle(handle),
            ) => self.handle_denial_outcome(handle),
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
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    self.query.name(),
                );
                self.authorization_denied_outcome(denial)
            }
            _ => WorthQueryApplicationLiveOutcome::Unavailable,
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
                _ => WorthQueryApplicationLiveOutcome::Unavailable,
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
        if let WorthQueryOperationAuthorizationDenialKind::Handle(handle)
        | WorthQueryOperationAuthorizationDenialKind::ProductSecurityBasis(
            crate::basis::WorthQueryProductBranchAdmissionDenial::Handle(handle),
        ) = denial.kind()
        {
            return self.handle_denial_outcome(handle);
        }
        self.acknowledge_and_terminate(WorthQueryApplicationLiveOutcome::AuthorizationDenied(
            Box::new(denial),
        ))
    }

    pub(super) fn handle_denial_outcome(
        &mut self,
        handle: crate::facade::primary_graph::WorthQueryHandleDenial,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled);
        WorthQueryApplicationLiveOutcome::AuthorizationDenied(Box::new(
            WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Handle(handle),
                self.query.name(),
            ),
        ))
    }

    fn cancelled_outcome(&mut self) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        if self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
            WorthQueryApplicationLiveOutcome::Cancelled
        } else {
            WorthQueryApplicationLiveOutcome::Unavailable
        }
    }

    fn deadline_exceeded_outcome(
        &mut self,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        if self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
            WorthQueryApplicationLiveOutcome::DeadlineExceeded
        } else {
            WorthQueryApplicationLiveOutcome::Unavailable
        }
    }
}
