//! Strict per-delivery re-admission, authority-watermark-bounded metered
//! commit-batch polling, and footprint currentness verification.
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_runtime::facade::ports::{
    AdvancementPhaseInternalPort as _, ProductBranchLeaseInternalPort as _,
    ProductObservationLeaseInternalPort as _, ProductRuntimeBranchLifecycleInternalPort as _,
};
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::super::outcome::WorthQueryApplicationLiveOverflow;
use super::footprint::WholeResultFootprintStatus;
use super::{
    WorthQueryApplicationWholeResultLiveLease, WorthQueryApplicationWholeResultLiveOutcome,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::application_query::{
    authorized_read::{execute_authorized_read, refresh_governed_authorization},
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryControls,
};
use crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveBoundedPoll;
use crate::domain_computation::primary_graph::{
    WorthQueryAuthenticatedPrincipal, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};

const MAX_HEAD_RECHECK_ATTEMPTS: usize = 8;

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
        let watermark = match self.admit_head_and_authority(execution, principal) {
            Ok(watermark) => watermark,
            Err(outcome) => return outcome,
        };
        self.poll_until_invalidated(execution, principal, request, watermark)
    }

    pub(super) fn admit_head_and_authority(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    ) -> Result<
        CompositeCommitIdentity,
        WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult>,
    > {
        let current_branch = self
            .runtime
            .runtime
            .product_runtime
            .admit_product_branch(&self.opening_branch)
            .map_err(|cause| self.handle_basis_admission_denial(cause))?;
        if current_branch.observation().lifecycle_incarnation() != self.opening_incarnation {
            return Err(
                if self.terminate(BridgeExecutionBasisTerminalDisposition::Completed) {
                    WorthQueryApplicationWholeResultLiveOutcome::Closed
                } else {
                    WorthQueryApplicationWholeResultLiveOutcome::Unavailable
                },
            );
        }
        let watermark = current_branch.observation().selected_commit().clone();
        self.admit_current_authority(execution, current_branch.read_lease(), principal)?;
        Ok(watermark)
    }

    /// Re-observes World head before returning `Pending`, `Overflow`, or
    /// `Delivered`. If World moved past `watermark`, re-runs full authority
    /// admission at the new head so concurrent revocations fail closed immediately.
    pub(super) fn confirm_authority_at_head(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        watermark: &mut CompositeCommitIdentity,
    ) -> Result<bool, WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult>> {
        let current_branch = self
            .runtime
            .runtime
            .product_runtime
            .admit_product_branch(&self.opening_branch)
            .map_err(|cause| self.handle_basis_admission_denial(cause))?;
        if current_branch.observation().lifecycle_incarnation() != self.opening_incarnation {
            return Err(
                if self.terminate(BridgeExecutionBasisTerminalDisposition::Completed) {
                    WorthQueryApplicationWholeResultLiveOutcome::Closed
                } else {
                    WorthQueryApplicationWholeResultLiveOutcome::Unavailable
                },
            );
        }
        let current_commit = current_branch.observation().selected_commit();
        if current_commit == watermark {
            return Ok(false);
        }
        let next_watermark = current_commit.clone();
        self.admit_current_authority(execution, current_branch.read_lease(), principal)?;
        *watermark = next_watermark;
        Ok(true)
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
        mut watermark: CompositeCommitIdentity,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        let mut recheck_attempts = 0_usize;
        loop {
            if let Some(outcome) = self.observe_interruption(request) {
                return outcome;
            }
            let poll_result = {
                let Some(subscription) = self.subscription.as_ref() else {
                    return WorthQueryApplicationWholeResultLiveOutcome::Closed;
                };
                self.runtime
                    .runtime
                    .primary_provider
                    .live_delivery_ref()
                    .poll_funded_bounded(
                        execution,
                        subscription,
                        self.cursor,
                        self.controls.buffer_capacity(),
                        &watermark,
                    )
            };
            let poll = match poll_result {
                Ok(poll) => poll,
                Err(denial) => return self.handle_live_admission_denial(denial),
            };
            let batch_cell = match poll {
                WorthQueryLiveBoundedPoll::Batch(batch) => batch,
                WorthQueryLiveBoundedPoll::HeadAdvanced => {
                    recheck_attempts += 1;
                    if recheck_attempts > MAX_HEAD_RECHECK_ATTEMPTS {
                        return self.terminate_outcome(
                            WorthQueryApplicationWholeResultLiveOutcome::Unavailable,
                        );
                    }
                    watermark = match self.admit_head_and_authority(execution, principal) {
                        Ok(next) => next,
                        Err(outcome) => return outcome,
                    };
                    continue;
                }
                WorthQueryLiveBoundedPoll::Pending => {
                    match self.confirm_authority_at_head(execution, principal, &mut watermark) {
                        Ok(false) => {
                            return WorthQueryApplicationWholeResultLiveOutcome::Pending;
                        }
                        Ok(true) => {
                            recheck_attempts += 1;
                            if recheck_attempts > MAX_HEAD_RECHECK_ATTEMPTS {
                                return self.terminate_outcome(
                                    WorthQueryApplicationWholeResultLiveOutcome::Unavailable,
                                );
                            }
                            continue;
                        }
                        Err(outcome) => return outcome,
                    }
                }
                WorthQueryLiveBoundedPoll::Overflow { missed } => {
                    if let Err(outcome) =
                        self.confirm_authority_at_head(execution, principal, &mut watermark)
                    {
                        return outcome;
                    }
                    return self.terminate_outcome(
                        WorthQueryApplicationWholeResultLiveOutcome::Overflow(
                            WorthQueryApplicationLiveOverflow::new(missed),
                        ),
                    );
                }
                WorthQueryLiveBoundedPoll::Closed => {
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
                watermark,
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
        let status = {
            let Some(footprint) = self.footprint.as_ref() else {
                let _ = basis.release();
                return Err(WorthQueryApplicationWholeResultLiveOutcome::Closed);
            };
            self.runtime
                .runtime
                .primary_provider
                .graph_ref()
                .with_runtime(execution, |relational| {
                    footprint.verify_at_snapshot(
                        execution,
                        relational,
                        basis.snapshot_handle(),
                        request,
                    )
                })
        };
        let released = basis.release().released();
        if !released {
            return Err(
                self.terminate_outcome(WorthQueryApplicationWholeResultLiveOutcome::Unavailable)
            );
        }
        status.map_err(|stop| self.handle_footprint_stop(stop))
    }
}
