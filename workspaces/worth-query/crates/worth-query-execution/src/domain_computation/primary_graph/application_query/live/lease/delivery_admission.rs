use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::{
    application_query::ApplicationQueryLiveCauseBinding, application_schema::ApplicationSchema,
};
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;

use super::WorthQueryApplicationLiveLease;
use crate::domain_computation::primary_graph::{
    application_query::{
        authorized_read::{execute_authorized_read, refresh_governed_authorization},
        live::scope_identity::read_scope_identity,
        WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
        WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
        WorthQueryApplicationQueryControls,
    },
    WorthQueryAuthenticatedPrincipal, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
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
    pub(super) fn admit_delivery_progress(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    ) -> Result<(), super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult>>
    {
        let product = self._opening_product.read_lease();
        let application_basis = self
            .runtime
            .retain_product_application_basis(execution, product.observation())
            .map_err(|cause| match cause {
                crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Ok(kind)) => {
                    self.execution_denied_outcome(kind)
                }
                crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
                )) => self.cancelled_outcome(),
                crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
                )) => self.deadline_exceeded_outcome(),
                crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => {
                    super::super::outcome::WorthQueryApplicationLiveOutcome::Unavailable
                }
            })?;
        let access = WorthQueryApplicationQueryAccessContext::new(principal, &self.scope);
        let controls = WorthQueryApplicationQueryControls::product_live(
            product,
            application_basis,
            self.controls.maximum_materialized_record_count(),
            self.controls.request(),
        );
        let governance = std::mem::replace(
            &mut self.governance,
            super::super::super::disclosure::WorthQueryApplicationQueryGovernance::Public,
        );
        let mut plan = match self.runtime.readmit_application_query_live(
            execution,
            &self.query,
            &access,
            governance,
            self.parameters.clone(),
            controls,
        ) {
            Ok(plan) => plan,
            Err(denial) => return Err(self.delivery_admission_denial(denial)),
        };
        if let Err(denial) = refresh_governed_authorization(execution, self.runtime, &mut plan) {
            let _ = plan.basis.release();
            return Err(self.delivery_read_denial(denial));
        }
        let ((scope_identity, observed), _, read_proof) =
            match execute_authorized_read(execution, self.runtime, &plan, |runtime, graph, plan| {
                read_scope_identity(execution, runtime, graph, plan)
            }) {
                Ok(read) => read,
                Err(denial) => {
                    let _ = plan.basis.release();
                    return Err(self.delivery_read_denial(denial));
                }
            };
        if scope_identity != self.scope_identity {
            let _ = plan.basis.release();
            return Err(self.terminate_delivery_outcome(
                super::super::outcome::WorthQueryApplicationLiveOutcome::StaleScope,
            ));
        }
        let governance = plan.take_governance();
        let basis_release = plan.basis.release();
        if !basis_release.released()
            || plan
                .graph_work
                .complete_query_read(read_proof, observed, basis_release)
                .is_err()
        {
            return Err(self.terminate_delivery_outcome(
                super::super::outcome::WorthQueryApplicationLiveOutcome::Unavailable,
            ));
        }
        self.governance = governance;
        Ok(())
    }

    fn delivery_admission_denial(
        &mut self,
        denial: WorthQueryApplicationQueryAdmissionDenial,
    ) -> super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        use super::super::outcome::WorthQueryApplicationLiveOutcome as Outcome;

        match denial.kind() {
            WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(kind) => {
                self.execution_denied_outcome(kind)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::Cancelled => {
                self.terminate_delivery_outcome(Outcome::Cancelled)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded => {
                self.terminate_delivery_outcome(Outcome::DeadlineExceeded)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal
            | WorthQueryApplicationQueryAdmissionDenialKind::ForeignPrincipal => {
                self.terminate_delivery_outcome(Outcome::StalePrincipal)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::StaleScope
            | WorthQueryApplicationQueryAdmissionDenialKind::ForeignScope
            | WorthQueryApplicationQueryAdmissionDenialKind::ScopeTypeMismatch => {
                self.terminate_delivery_outcome(Outcome::StaleScope)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_) => {
                let authorization = denial.into_authorization_denial().unwrap_or_else(|| {
                    WorthQueryOperationAuthorizationDenial::inconsistent(self.query.name())
                });
                self.delivery_authorization_denial(authorization)
            }
            WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
            | WorthQueryApplicationQueryAdmissionDenialKind::DisclosureAuthorizationMismatch
            | WorthQueryApplicationQueryAdmissionDenialKind::InternalComputationDenied => self
                .terminate_delivery_outcome(Outcome::AuthorizationDenied(Box::new(
                    WorthQueryOperationAuthorizationDenial::new(
                        WorthQueryOperationAuthorizationDenialKind::Refused(
                            AuthorizationRefusal::InconsistentDecision,
                        ),
                        self.query.name(),
                    ),
                ))),
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
            | WorthQueryApplicationQueryAdmissionDenialKind::ExecutionShapeUnsupported => self.terminate_delivery_outcome(Outcome::Unavailable),
        }
    }

    fn delivery_read_denial(
        &mut self,
        denial: crate::domain_computation::primary_graph::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial,
    ) -> super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        use super::super::outcome::WorthQueryApplicationLiveOutcome as Outcome;
        use crate::domain_computation::primary_graph::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial as Denial;

        match denial {
            Denial::StalePrincipal => self.terminate_delivery_outcome(Outcome::StalePrincipal),
            Denial::StaleScope | Denial::StaleBasisScope => {
                self.terminate_delivery_outcome(Outcome::StaleScope)
            }
            Denial::Authorization(authorization) => {
                self.delivery_authorization_denial(authorization)
            }
            Denial::Read(read) => {
                use crate::domain_computation::primary_graph::application_query::read_execution::WorthQueryApplicationReadExecutionDenialKind as Read;
                let outcome = match read.kind() {
                    Read::ExecutionRequest(kind) => Outcome::ExecutionRequest(kind),
                    Read::Cancelled => Outcome::Cancelled,
                    Read::DeadlineExceeded => Outcome::DeadlineExceeded,
                    Read::PredicateIndexUnavailable
                    | Read::PredicateLookupOverflow
                    | Read::TargetIdentityIndexUnavailable
                    | Read::TargetIdentityLookupOverflow
                    | Read::TargetIdentityNotFound
                    | Read::ResultLimitExceeded
                    | Read::CardinalityMismatch
                    | Read::TraversalUnavailable
                    | Read::ContinuationIndexUnavailable
                    | Read::ContinuationBoundaryRejected
                    | Read::ContinuationGenerationChanged
                    | Read::ContinuationPageWidthInvalid
                    | Read::ProjectionUnavailable
                    | Read::ResultBufferLimitExceeded
                    | Read::WorkLimitExceeded => Outcome::Unavailable,
                };
                self.terminate_delivery_outcome(outcome)
            }
            Denial::Session => self.terminate_delivery_outcome(Outcome::Unavailable),
        }
    }

    fn delivery_authorization_denial(
        &mut self,
        denial: WorthQueryOperationAuthorizationDenial,
    ) -> super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        use super::super::outcome::WorthQueryApplicationLiveOutcome as Outcome;
        let outcome = match denial.kind() {
            WorthQueryOperationAuthorizationDenialKind::Stopped(Ok(kind)) => {
                Outcome::ExecutionRequest(kind)
            }
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
            )) => Outcome::Cancelled,
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
            )) => Outcome::DeadlineExceeded,
            WorthQueryOperationAuthorizationDenialKind::Refused(_) => {
                Outcome::AuthorizationDenied(Box::new(denial))
            }
        };
        self.terminate_delivery_outcome(outcome)
    }

    fn terminate_delivery_outcome(
        &mut self,
        outcome: super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult>,
    ) -> super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        use super::super::outcome::WorthQueryApplicationLiveOutcome as Outcome;
        match outcome {
            stop @ (Outcome::ExecutionRequest(_)
            | Outcome::Cancelled
            | Outcome::DeadlineExceeded) => {
                self.terminate_stopped_read();
                stop
            }
            ordinary @ (Outcome::Delivered(_)
            | Outcome::Pending
            | Outcome::Overflow(_)
            | Outcome::AuthorizationDenied(_)
            | Outcome::StalePrincipal
            | Outcome::StaleScope
            | Outcome::ProjectionDenied(_)
            | Outcome::CauseDenied(_)
            | Outcome::Closed
            | Outcome::Unavailable) => {
                if self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
                    ordinary
                } else {
                    Outcome::Unavailable
                }
            }
        }
    }
}
