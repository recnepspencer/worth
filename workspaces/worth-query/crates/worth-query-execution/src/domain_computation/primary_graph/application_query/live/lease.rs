use std::marker::PhantomData;
use std::rc::Rc;

use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
#[cfg(test)]
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryLiveCauseBinding, ApplicationQueryParameterSet},
    application_schema::{
        ApplicationScalarValueBinding, ApplicationSchema, ApplicationStructuredValueBinding,
    },
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;

mod denial;
mod lifecycle;
mod open;
mod validation;

use super::{
    controls::WorthQueryApplicationLiveControls,
    outcome::{
        WorthQueryApplicationLiveCauseDenialKind, WorthQueryApplicationLiveCloseOutcome,
        WorthQueryApplicationLiveOutcome, WorthQueryApplicationLiveOverflow,
        WorthQueryApplicationLiveUpdate,
    },
    projection::{finalize_live_projection, WorthQueryLiveProjectionFinalizationDenial},
};
use crate::domain_computation::{
    managed_run::WorthQueryManagedLowerExecutionBasis,
    primary_graph::{
        application_query::{
            authorized_read::{execute_authorized_read, refresh_governed_authorization},
            read_execution::read_live_target,
            WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
            WorthQueryApplicationQueryControls,
        },
        live_delivery::{WorthQueryLiveCauseFillPosture, WorthQueryLiveCauseQueue},
        WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
};

pub struct WorthQueryApplicationLiveLease<
    'runtime,
    'principal,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Target,
    Binding,
> where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    principal: &'principal WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
    parameters: ApplicationQueryParameterSet<Query>,
    controls: WorthQueryApplicationLiveControls,
    _opening_product: crate::basis::WorthQueryProductBranchLease,
    governance: super::super::disclosure::WorthQueryApplicationQueryGovernance,
    scope_identity: AspectValue,
    basis: Option<WorthQueryManagedLowerExecutionBasis>,
    graph_work:
        Option<crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession>,
    read_proof:
        Option<crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof>,
    initial_read_work:
        Option<crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork>,
    basis_release: Option<super::super::WorthQueryApplicationBasisReleaseReceipt>,
    read_completion:
        Option<crate::domain_computation::provider_session::WorthQueryGraphReadCompletion>,
    queue: WorthQueryLiveCauseQueue<
        <Binding::PayloadBinding as ApplicationStructuredValueBinding>::Value,
    >,
    _target: PhantomData<fn() -> (Target, Binding)>,
    _thread_affinity: PhantomData<Rc<()>>,
}

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
    pub fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.graph_work
            .as_ref()
            .expect("an open live lease retains its graph-work session")
            .identity()
    }

    pub fn buffered_cause_count(&self) -> usize {
        self.queue.buffered_cause_count()
    }

    pub fn poll(&mut self) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        if self.basis.is_none() {
            return WorthQueryApplicationLiveOutcome::Closed;
        }
        if let Some(interruption) = self.controls.request().interruption() {
            if !self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
                return WorthQueryApplicationLiveOutcome::Unavailable;
            }
            return match interruption {
                WorthQueryRequestInterruption::Cancelled => {
                    WorthQueryApplicationLiveOutcome::Cancelled
                }
                WorthQueryRequestInterruption::DeadlineExceeded => {
                    WorthQueryApplicationLiveOutcome::DeadlineExceeded
                }
            };
        }
        let fill = {
            let Some(basis) = self.basis.as_mut() else {
                return WorthQueryApplicationLiveOutcome::Closed;
            };
            let expected_scope = &self.scope_identity;
            self.queue.fill(
                &self.runtime.primary_provider.live_delivery,
                basis,
                Binding::effect(),
                self.controls.buffer_capacity(),
                |payload| {
                    Binding::ScopeIdentityBinding::encode(&Binding::scope_identity(payload))
                        .is_ok_and(|identity| identity == *expected_scope)
                },
            )
        };
        let terminal = match fill {
            WorthQueryLiveCauseFillPosture::Pending => WorthQueryApplicationLiveOutcome::Pending,
            WorthQueryLiveCauseFillPosture::Overflow(missed) => {
                WorthQueryApplicationLiveOutcome::Overflow(WorthQueryApplicationLiveOverflow::new(
                    missed,
                ))
            }
            WorthQueryLiveCauseFillPosture::Closed => WorthQueryApplicationLiveOutcome::Closed,
            WorthQueryLiveCauseFillPosture::Unavailable => {
                WorthQueryApplicationLiveOutcome::Unavailable
            }
        };
        let Some((publication, product, payload)) = self.queue.front(&Binding::effect()) else {
            return match terminal {
                WorthQueryApplicationLiveOutcome::Overflow(overflow) => {
                    if self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
                        WorthQueryApplicationLiveOutcome::Overflow(overflow)
                    } else {
                        WorthQueryApplicationLiveOutcome::Unavailable
                    }
                }
                WorthQueryApplicationLiveOutcome::Closed => {
                    if self.terminate(BridgeExecutionBasisTerminalDisposition::Completed) {
                        WorthQueryApplicationLiveOutcome::Closed
                    } else {
                        WorthQueryApplicationLiveOutcome::Unavailable
                    }
                }
                outcome => outcome,
            };
        };
        let target_identity =
            match Binding::TargetIdentityBinding::encode(&Binding::target_identity(payload)) {
                Ok(identity) => identity,
                Err(_) => return WorthQueryApplicationLiveOutcome::Unavailable,
            };
        self.project_front(
            publication.clone(),
            product.retained_clone(),
            target_identity,
        )
    }

    fn project_front(
        &mut self,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        product: crate::basis::WorthQueryProductObservationLease,
        target_identity: AspectValue,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        let access = WorthQueryApplicationQueryAccessContext::new(self.principal, &self.scope);
        let application_basis = match self
            .runtime
            .retain_product_application_basis(product.observation())
        {
            Ok(basis) => basis,
            Err(_) => return WorthQueryApplicationLiveOutcome::Unavailable,
        };
        let controls = WorthQueryApplicationQueryControls::product_live(
            product,
            application_basis,
            self.controls.maximum_materialized_record_count(),
            self.controls.maximum_work_per_delivery(),
            self.controls.request(),
        );
        let governance = std::mem::replace(
            &mut self.governance,
            super::super::disclosure::WorthQueryApplicationQueryGovernance::Public,
        );
        let mut plan = match self.runtime.readmit_application_query_live(
            &self.query,
            &access,
            governance,
            self.parameters.clone(),
            controls,
        ) {
            Ok(plan) => plan,
            Err(denial) => return self.handle_admission_denial(denial),
        };
        if let Err(denial) = refresh_governed_authorization(self.runtime, &mut plan) {
            let _ = plan.basis.release();
            return self.handle_read_denial(denial);
        }
        let Some(_) = self.runtime.runtime.primary_graph() else {
            let _ = plan.basis.release();
            return WorthQueryApplicationLiveOutcome::Unavailable;
        };
        let result_buffer = self.runtime.result_buffers.reserve(
            plan.graph_read_plan()
                .budget_check()
                .max_inline_result_bytes(),
        );
        let (raw, authorization_work, read_proof) =
            match execute_authorized_read(self.runtime, &plan, |runtime, graph, plan| {
                read_live_target(runtime, graph, plan, target_identity, result_buffer)
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
        match finalize_live_projection(plan, raw, authorization_work, read_proof) {
            Ok((result, receipt, governance)) => {
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
                if self.acknowledge_front() {
                    WorthQueryApplicationLiveOutcome::ProjectionDenied(kind)
                } else {
                    WorthQueryApplicationLiveOutcome::Unavailable
                }
            }
        }
    }

    fn acknowledge_cause_denial(
        &mut self,
        kind: WorthQueryApplicationLiveCauseDenialKind,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        self.acknowledge_and_terminate(WorthQueryApplicationLiveOutcome::CauseDenied(kind))
    }

    fn acknowledge_front(&mut self) -> bool {
        self.basis
            .as_mut()
            .is_some_and(|basis| self.queue.acknowledge_front(basis).is_ok())
    }

    fn acknowledge_and_terminate(
        &mut self,
        outcome: WorthQueryApplicationLiveOutcome<Query, QueryResult>,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        if self.acknowledge_front()
            && self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled)
        {
            outcome
        } else {
            WorthQueryApplicationLiveOutcome::Unavailable
        }
    }

    pub fn close(mut self) -> WorthQueryApplicationLiveCloseOutcome {
        if self.terminate(BridgeExecutionBasisTerminalDisposition::Completed) {
            self.read_completion.take().map_or(
                WorthQueryApplicationLiveCloseOutcome::Unavailable,
                WorthQueryApplicationLiveCloseOutcome::Completed,
            )
        } else {
            WorthQueryApplicationLiveCloseOutcome::Unavailable
        }
    }

    /// Test harness only: bind an already-settled request scope to the open
    /// lease so deadline/cancellation poll outcomes are deterministic.
    #[cfg(test)]
    pub(crate) fn replace_request(&mut self, request: WorthQueryRequestScope) {
        self.controls.replace_request(request);
    }
}

#[cfg(test)]
#[path = "lease/delivery_tests.rs"]
mod delivery_tests;
#[cfg(test)]
#[path = "lease/reservation_race_tests.rs"]
mod reservation_race_tests;
#[cfg(test)]
#[path = "lease/sibling_delivery_tests.rs"]
mod sibling_delivery_tests;
