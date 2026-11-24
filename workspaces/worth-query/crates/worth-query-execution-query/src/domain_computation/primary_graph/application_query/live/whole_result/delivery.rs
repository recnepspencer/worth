//! Strict per-delivery re-admission, metered commit-batch polling, footprint
//! currentness verification, and whole-result refresh execution.
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::{
    AdvancementPhaseInternalPort as _, ProductBranchLeaseInternalPort as _,
    ProductObservationLeaseInternalPort as _, ProductRuntimeBranchLifecycleInternalPort as _,
};
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;

use super::super::outcome::WorthQueryApplicationLiveOverflow;
use super::footprint::{WholeResultFootprintStatus, WholeResultInvalidationFootprint};
use super::{
    WorthQueryApplicationWholeResultLiveLease, WorthQueryApplicationWholeResultLiveOutcome,
    WorthQueryApplicationWholeResultLiveSnapshot, WorthQueryApplicationWholeResultLiveUpdate,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::application_query::{
    authorized_read::{execute_authorized_read, refresh_governed_authorization},
    one_shot::outcome::complete_one_shot,
    read_execution::{persistent_names::observe_persistent_names, read_bounded_root_rows},
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryControls,
};
use crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveSourcePoll;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use crate::domain_computation::primary_graph::{
    WorthQueryAuthenticatedPrincipal, WorthQueryCommittedProductPublication,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

impl<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryApplicationWholeResultLiveLease<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    pub fn next(
        &mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        request: &WorthQueryRequestScope,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        let phase_request = match phase.execution_request_for(&self.runtime.runtime.product_runtime)
        {
            Ok(request) => request,
            Err(_) => {
                return WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(Box::new(
                    WorthQueryOperationAuthorizationDenial::new(
                        WorthQueryOperationAuthorizationDenialKind::Refused(
                            AuthorizationRefusal::ForeignPhase,
                        ),
                        "phase-custody",
                    ),
                ))
            }
        };
        let execution = &phase_request;
        if let Some(outcome) = self.observe_interruption(request) {
            return outcome;
        }
        self.controls.replace_request(request.clone());
        let opening_observation = self._opening_product.observation();
        let current_branch = match self
            .runtime
            .runtime
            .product_runtime
            .admit_product_branch(opening_observation.branch_identity())
        {
            Ok(branch) => branch,
            Err(cause) => return self.handle_basis_admission_denial(cause),
        };
        if current_branch.observation().lifecycle_incarnation()
            != opening_observation.lifecycle_incarnation()
        {
            return if self.terminate(BridgeExecutionBasisTerminalDisposition::Completed) {
                WorthQueryApplicationWholeResultLiveOutcome::Closed
            } else {
                WorthQueryApplicationWholeResultLiveOutcome::Unavailable
            };
        }
        // Strict re-admission against CURRENT branch truth BEFORE polling or
        // delivering any queued commit batch: revoked principals/grants fail
        // closed immediately with zero leaked rows.
        if let Err(outcome) =
            self.admit_current_authority(execution, current_branch.read_lease(), principal)
        {
            return outcome;
        }
        self.poll_until_invalidated(execution, principal, request)
    }

    fn admit_current_authority(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        current_product: crate::basis::WorthQueryProductObservationLease,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    ) -> Result<(), WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult>> {
        let application_basis = self
            .runtime
            .retain_product_application_basis(execution, current_product.observation())
            .map_err(|cause| self.handle_basis_admission_denial(cause))?;
        let controls = WorthQueryApplicationQueryControls::product_live(
            current_product,
            application_basis,
            self.controls.maximum_materialized_record_count(),
            self.controls.request(),
        );
        let governance = std::mem::replace(
            &mut self.governance,
            crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance::Public,
        );
        let mut plan = {
            let access = WorthQueryApplicationQueryAccessContext::new(principal, &self.scope);
            match self.runtime.readmit_application_query_live(
                execution,
                &self.query,
                &access,
                governance,
                self.parameters.clone(),
                controls,
            ) {
                Ok(plan) => plan,
                Err(denial) => return Err(self.handle_admission_denial(denial)),
            }
        };
        if let Err(denial) = refresh_governed_authorization(execution, self.runtime, &mut plan) {
            let _ = plan.basis.release();
            return Err(self.handle_read_denial(denial));
        }
        let (observed, _, read_proof) =
            match execute_authorized_read(execution, self.runtime, &plan, |_, _, _| {
                Ok(
                    crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork {
                        examined_candidates: 0,
                        projected_records: 0,
                        projected_fields: 0,
                        relation_records_examined: 0,
                        ordering_comparisons: 0,
                    },
                )
            }) {
                Ok(ok) => ok,
                Err(denial) => {
                    let _ = plan.basis.release();
                    return Err(self.handle_read_denial(denial));
                }
            };
        let governance = plan.take_governance();
        let basis_release = plan.basis.release();
        if !basis_release.released()
            || plan
                .graph_work
                .complete_query_read(read_proof, observed, basis_release)
                .is_err()
        {
            return Err(
                self.terminate_outcome(WorthQueryApplicationWholeResultLiveOutcome::Unavailable)
            );
        }
        self.governance = governance;
        Ok(())
    }

    fn poll_until_invalidated(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        request: &WorthQueryRequestScope,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        let source = self.runtime.runtime.primary_provider.live_delivery_ref();
        loop {
            if let Some(outcome) = self.observe_interruption(request) {
                return outcome;
            }
            if let Err(cause) = SourceFactsAllocation::work(execution, 1) {
                return self.handle_relational_cause(cause);
            }
            let batch_cell = match source.poll(&self.subscription, self.cursor) {
                WorthQueryLiveSourcePoll::Batch(batch) => batch,
                WorthQueryLiveSourcePoll::Pending => {
                    return WorthQueryApplicationWholeResultLiveOutcome::Pending
                }
                WorthQueryLiveSourcePoll::Overflow { missed } => {
                    return self.terminate_outcome(
                        WorthQueryApplicationWholeResultLiveOutcome::Overflow(
                            WorthQueryApplicationLiveOverflow::new(missed),
                        ),
                    );
                }
                WorthQueryLiveSourcePoll::Closed => {
                    return if self.terminate(BridgeExecutionBasisTerminalDisposition::Completed) {
                        WorthQueryApplicationWholeResultLiveOutcome::Closed
                    } else {
                        WorthQueryApplicationWholeResultLiveOutcome::Unavailable
                    };
                }
            };
            let batch = batch_cell.batch();
            let next_cursor = match batch.sequence.checked_add(1) {
                Some(next) => next,
                None => {
                    return self.terminate_outcome(
                        WorthQueryApplicationWholeResultLiveOutcome::Unavailable,
                    );
                }
            };
            let status = match self.verify_batch_footprint(execution, batch.product(), request) {
                Ok(status) => status,
                Err(outcome) => return outcome,
            };
            if status == WholeResultFootprintStatus::Current {
                self.cursor = next_cursor;
                continue;
            }
            return self.refresh_and_deliver(
                execution,
                principal,
                batch.publication().clone(),
                batch.product().retained_clone(),
                next_cursor,
            );
        }
    }

    fn verify_batch_footprint(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        product: &crate::basis::WorthQueryProductObservationLease,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WholeResultFootprintStatus,
        WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult>,
    > {
        let basis = self
            .runtime
            .retain_product_application_basis(execution, product.observation())
            .map_err(|cause| self.handle_basis_admission_denial(cause))?;
        let status = self
            .runtime
            .runtime
            .primary_provider
            .graph_ref()
            .with_runtime(execution, |relational| {
                self.footprint.verify_at_snapshot(
                    execution,
                    relational,
                    basis.snapshot_handle(),
                    request,
                )
            });
        let released = basis.release().released();
        if !released {
            return Err(
                self.terminate_outcome(WorthQueryApplicationWholeResultLiveOutcome::Unavailable)
            );
        }
        status.map_err(|stop| self.handle_footprint_stop(stop))
    }

    fn refresh_and_deliver(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        publication: WorthQueryCommittedProductPublication,
        product: crate::basis::WorthQueryProductObservationLease,
        next_cursor: u64,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        let application_basis = match self
            .runtime
            .retain_product_application_basis(execution, product.observation())
        {
            Ok(basis) => basis,
            Err(cause) => return self.handle_basis_admission_denial(cause),
        };
        let access = WorthQueryApplicationQueryAccessContext::new(principal, &self.scope);
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
        let (raw, authorization_work, read_proof) =
            match execute_authorized_read(execution, self.runtime, &plan, |runtime, graph, plan| {
                let mut raw = read_bounded_root_rows(runtime, execution, graph, plan, None)?;
                if QueryResult::PROJECTS_PERSISTENT_NAMES {
                    observe_persistent_names(runtime, execution, plan, &mut raw);
                }
                Ok(raw)
            }) {
                Ok(ok) => ok,
                Err(denial) => {
                    let released = plan.basis.release().released();
                    if !released {
                        return WorthQueryApplicationWholeResultLiveOutcome::Unavailable;
                    }
                    return self.handle_read_denial(denial);
                }
            };
        let completed = match complete_one_shot(
            execution,
            self.runtime,
            plan,
            raw,
            authorization_work,
            read_proof,
        ) {
            Ok(completed) => completed,
            Err(denial) => return self.handle_one_shot_denial(denial),
        };
        let layout = self
            .runtime
            .runtime
            .primary_provider
            .graph_ref()
            .layout_ref();
        let next_footprint = match WholeResultInvalidationFootprint::capture(
            execution,
            layout,
            self.query.name(),
            self.controls.request(),
            completed.read.result_set_observation(),
            completed.read.observed_sources(),
        ) {
            Ok(footprint) => footprint,
            Err(stop) => return self.handle_footprint_stop(stop),
        };
        let request = self.controls.request().clone();
        if let Some(outcome) = self.observe_interruption(&request) {
            return outcome;
        }
        self.governance = completed.governance;
        self.footprint = next_footprint;
        self.cursor = next_cursor;
        let (rows, observed_sources, result_set_observation, receipt) =
            completed.read.into_whole_result_parts();
        let snapshot = WorthQueryApplicationWholeResultLiveSnapshot::new(
            rows,
            observed_sources,
            result_set_observation,
            receipt,
        );
        WorthQueryApplicationWholeResultLiveOutcome::Delivered(
            WorthQueryApplicationWholeResultLiveUpdate::new(publication, snapshot),
        )
    }
}
