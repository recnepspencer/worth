//! One queued native delivery retains decode custody through its final ACK.
use super::*;

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
    pub(super) fn project_front(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        product: crate::basis::WorthQueryProductObservationLease,
        target_identity: AspectValue,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        let access = WorthQueryApplicationQueryAccessContext::new(principal, &self.scope);
        let application_basis = match self
            .runtime
            .retain_product_application_basis(execution, product.observation())
        {
            Ok(basis) => basis,
            Err(cause) => {
                return match basis_interruption::outcome(cause) {
                    Some(outcome) => {
                        self.terminate_stopped_read();
                        outcome
                    }
                    None => WorthQueryApplicationLiveOutcome::Unavailable,
                }
            }
        };
        let controls = WorthQueryApplicationQueryControls::product_live(
            product,
            application_basis,
            self.controls.maximum_materialized_record_count(),
            self.controls.request(),
        );
        let governance = std::mem::replace(
            &mut self.governance,
            crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance::Public,
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
            Err(denial) => return self.handle_admission_denial(denial),
        };
        if let Err(denial) = refresh_governed_authorization(execution, self.runtime, &mut plan) {
            let _ = plan.basis.release();
            return self.handle_read_denial(denial);
        }
        let Some(_) = self.runtime.runtime.primary_graph() else {
            let _ = plan.basis.release();
            return WorthQueryApplicationLiveOutcome::Unavailable;
        };

        let (raw, authorization_work, read_proof) =
            match execute_authorized_read(execution, self.runtime, &plan, |runtime, graph, plan| {
                read_live_target(runtime, execution, graph, plan, target_identity)
            }) {
                Ok(raw) => raw,
                Err(denial) => {
                    let released = plan.basis.release().released();
                    if !released {
                        return WorthQueryApplicationLiveOutcome::Unavailable;
                    }
                    return self.handle_read_denial(denial);
                }
            };
        // This delivery owns parse custody through the native receipt AND the
        // queued cause acknowledgement. Projection cannot end that custody.
        let decode = crate::domain_computation::primary_graph::value_decode::ValueDecodeState::new(
            self.runtime.installed_schema().value_bindings().clone(),
        );
        match finalize_live_projection(
            execution,
            &decode,
            plan,
            raw,
            authorization_work,
            read_proof,
        ) {
            Ok((result, receipt, governance)) => {
                if let Err(cause) = decode.check(execution) {
                    return self.handle_projection_denial(
                        crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjectionDenialKind::ValueDecode(cause),
                    );
                }
                let request = self.controls.request().clone();
                if let Some(outcome) = self.observe_interruption(&request) {
                    return outcome;
                }
                self.governance = governance;
                if !self.acknowledge_front() {
                    return WorthQueryApplicationLiveOutcome::Unavailable;
                }
                WorthQueryApplicationLiveOutcome::Delivered(WorthQueryApplicationLiveUpdate::new(
                    publication,
                    result,
                    receipt,
                ))
            }
            Err(WorthQueryLiveProjectionFinalizationDenial::BasisRelease) => {
                WorthQueryApplicationLiveOutcome::Unavailable
            }
            Err(WorthQueryLiveProjectionFinalizationDenial::ResultShape) => self
                .acknowledge_cause_denial(
                    WorthQueryApplicationLiveCauseDenialKind::ResultShapeUnavailable,
                ),
            Err(WorthQueryLiveProjectionFinalizationDenial::Projection(kind)) => {
                self.handle_projection_denial(kind)
            }
        }
    }
}
