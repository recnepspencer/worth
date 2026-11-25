//! Whole-result refresh execution, footprint re-capture, and post-refresh
//! current-head authority confirmation before exposing a delivered snapshot.
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductObservationLeaseInternalPort as _;
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::footprint::WholeResultInvalidationFootprint;
use super::{
    WholeResultLiveStageForTest, WholeResultStageObserver,
    WorthQueryApplicationWholeResultLiveLease, WorthQueryApplicationWholeResultLiveOutcome,
    WorthQueryApplicationWholeResultLiveSnapshot, WorthQueryApplicationWholeResultLiveUpdate,
};
use crate::domain_computation::primary_graph::application_query::{
    authorized_read::{execute_authorized_read, refresh_governed_authorization},
    one_shot::outcome::complete_one_shot,
    read_execution::{persistent_names::observe_persistent_names, read_bounded_root_rows},
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryControls,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAuthenticatedPrincipal, WorthQueryCommittedProductPublication,
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
    #[allow(clippy::too_many_arguments)]
    pub(super) fn refresh_and_deliver<O: WholeResultStageObserver>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        publication: WorthQueryCommittedProductPublication,
        product: crate::basis::WorthQueryProductObservationLease,
        next_cursor: u64,
        mut watermark: CompositeCommitIdentity,
        mut observer: O,
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
        if let Err(denial) = refresh_governed_authorization(execution, &self.runtime, &mut plan) {
            let _ = plan.basis.release();
            return self.handle_read_denial(denial);
        }
        let (raw, authorization_work, read_proof) = match execute_authorized_read(
            execution,
            &self.runtime,
            &plan,
            |runtime, graph, plan| {
                let mut raw = read_bounded_root_rows(runtime, execution, graph, plan, None)?;
                if QueryResult::PROJECTS_PERSISTENT_NAMES {
                    observe_persistent_names(runtime, execution, plan, &mut raw);
                }
                Ok(raw)
            },
        ) {
            Ok(ok) => ok,
            Err(denial) => {
                let released = plan.basis.release().released();
                if !released {
                    return self.terminate_outcome(
                        WorthQueryApplicationWholeResultLiveOutcome::Unavailable,
                    );
                }
                return self.handle_read_denial(denial);
            }
        };
        let completed = match complete_one_shot(
            execution,
            &self.runtime,
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
        observer.on_stage(WholeResultLiveStageForTest::BeforeFootprintCapture);
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
        // Restore governance and confirm current World head authority BEFORE
        // publishing this snapshot: a revocation committed during refresh fails
        // closed with zero leaked rows and immediately retires source custody.
        self.governance = completed.governance;
        observer.on_stage(WholeResultLiveStageForTest::AfterFootprintCaptureBeforeDeliveryConfirm);
        if let Err(outcome) = self.confirm_authority_at_head(execution, principal, &mut watermark) {
            return outcome;
        }
        let request = self.controls.request().clone();
        if let Some(outcome) = self.observe_interruption(&request) {
            return outcome;
        }
        self.footprint = Some(next_footprint);
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
