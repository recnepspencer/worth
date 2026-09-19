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
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    ) -> Result<(), super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult>>
    {
        let access = WorthQueryApplicationQueryAccessContext::new(principal, &self.scope);
        let product = self._opening_product.read_lease();
        let application_basis = self
            .runtime
            .retain_product_application_basis(product.observation())
            .map_err(|_| super::super::outcome::WorthQueryApplicationLiveOutcome::Unavailable)?;
        let controls = WorthQueryApplicationQueryControls::product_live(
            product,
            application_basis,
            self.controls.maximum_materialized_record_count(),
            self.controls.maximum_work_per_delivery(),
            self.controls.request(),
        );
        let governance = std::mem::replace(
            &mut self.governance,
            super::super::super::disclosure::WorthQueryApplicationQueryGovernance::Public,
        );
        let mut plan = match self.runtime.readmit_application_query_live(
            &self.query,
            &access,
            governance,
            self.parameters.clone(),
            controls,
        ) {
            Ok(plan) => plan,
            Err(denial) => return Err(self.delivery_admission_denial(denial)),
        };
        if let Err(denial) = refresh_governed_authorization(self.runtime, &mut plan) {
            let _ = plan.basis.release();
            return Err(self.delivery_read_denial(denial));
        }
        let ((scope_identity, observed), _, read_proof) =
            match execute_authorized_read(self.runtime, &plan, read_scope_identity) {
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
                self.terminate_delivery_outcome(Outcome::AuthorizationDenied(Box::new(
                    authorization,
                )))
            }
            WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
            | WorthQueryApplicationQueryAdmissionDenialKind::DisclosureAuthorizationMismatch
            | WorthQueryApplicationQueryAdmissionDenialKind::InternalComputationDenied => self
                .terminate_delivery_outcome(Outcome::AuthorizationDenied(Box::new(
                    WorthQueryOperationAuthorizationDenial::new(
                        WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                        self.query.name(),
                    ),
                ))),
            _ => self.terminate_delivery_outcome(Outcome::Unavailable),
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
            Denial::Authorization(authorization) => self
                .terminate_delivery_outcome(Outcome::AuthorizationDenied(Box::new(authorization))),
            Denial::Read(_) | Denial::Session => {
                self.terminate_delivery_outcome(Outcome::Unavailable)
            }
        }
    }

    fn terminate_delivery_outcome(
        &mut self,
        outcome: super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult>,
    ) -> super::super::outcome::WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        if self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
            outcome
        } else {
            super::super::outcome::WorthQueryApplicationLiveOutcome::Unavailable
        }
    }
}
