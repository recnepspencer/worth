//! Opening a whole-result live subscription: metered commit-aligned subscription
//! open, initial authorized whole-result read, footprint capture, and managed
//! bridge basis admission.
use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryParameterSet, application_schema::ApplicationSchema,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::{
    AdvancementPhaseInternalPort as _, ProductBranchLeaseInternalPort as _,
    ProductRuntimeBranchLifecycleInternalPort as _,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::admit_live_managed_basis;
use super::super::controls::WorthQueryApplicationLiveControls;
use super::super::lease::validation::{open_admission_denial, open_denial, open_read_denial};
use super::super::outcome::{
    WorthQueryApplicationLiveOpenDenial, WorthQueryApplicationLiveOpenDenialKind,
};
use super::denial::{
    map_open_branch_admission_denial, map_open_footprint_stop, map_open_live_admission_denial,
    map_open_one_shot_denial,
};
use super::footprint::WholeResultInvalidationFootprint;
use super::{
    WorthQueryApplicationWholeResultLiveLease, WorthQueryApplicationWholeResultLiveSnapshot,
};
use crate::domain_computation::primary_graph::{
    application_query::{
        admission::prepare_governed_access,
        authorized_read::{execute_authorized_read, refresh_governed_authorization},
        disclosure::WorthQueryPendingApplicationQueryGovernance,
        one_shot::outcome::complete_one_shot,
        read_execution::{persistent_names::observe_persistent_names, read_bounded_root_rows},
        WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
        WorthQueryApplicationQueryControls,
    },
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime,
};

struct WholeResultLiveOpenRequest<
    'principal,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    principal: &'principal WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
    parameters: ApplicationQueryParameterSet<Query>,
    controls: WorthQueryApplicationLiveControls,
    product: crate::basis::WorthQueryProductBranchLease,
    application_basis: super::super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
    pending_governance: Option<WorthQueryPendingApplicationQueryGovernance>,
    producer: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::application_output_demand::OriginalSettledProducerObservation,
        >,
    >,
}

impl<'runtime, Schema>
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::too_many_arguments)]
    pub fn open_application_query_live_whole_result<
        'principal,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        principal: &'principal WorthQueryAuthenticatedPrincipal<
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationLiveControls,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveLease<
            'runtime,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationLiveOpenDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        let producer = self.original_settled_producer().cloned();
        let (application, product, application_basis) = self.into_parts();
        application.open_whole_result_live_with_governance(
            phase,
            WholeResultLiveOpenRequest {
                query,
                principal,
                scope,
                parameters,
                controls,
                product,
                application_basis,
                pending_governance: None,
                producer,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_governed_application_query_live_whole_result<
        'principal,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Capability,
        Operation,
        Input,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        principal: &'principal WorthQueryAuthenticatedPrincipal<
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
        capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        controls: WorthQueryApplicationLiveControls,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveLease<
            'runtime,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationLiveOpenDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
    {
        let producer = self.original_settled_producer().cloned();
        let (application, product, application_basis) = self.into_parts();
        let access = WorthQueryApplicationQueryAccessContext::new(principal, &scope);
        let pending =
            prepare_governed_access(application, &query, &access, capability, controls.request())
                .map_err(open_admission_denial)?;
        application.open_whole_result_live_with_governance(
            phase,
            WholeResultLiveOpenRequest {
                query,
                principal,
                scope,
                parameters,
                controls,
                product,
                application_basis,
                pending_governance: Some(pending),
                producer,
            },
        )
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn open_whole_result_live_with_governance<
        'runtime,
        'principal,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'runtime self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        request: WholeResultLiveOpenRequest<
            'principal,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveLease<
            'runtime,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationLiveOpenDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| {
                open_denial(
                    WorthQueryApplicationLiveOpenDenialKind::ForeignAdvancementPhase,
                    request.query.name(),
                )
            })?;
        if !request.query.is_live_whole_result() {
            return Err(open_denial(
                WorthQueryApplicationLiveOpenDenialKind::LiveContractMissing,
                request.query.name(),
            ));
        }
        let retained_product = request.product.retained_clone();
        let opening_branch = retained_product.observation().branch_identity().clone();
        let opening_incarnation = retained_product.observation().lifecycle_incarnation();
        // Open the commit-aligned live delivery subscription BEFORE executing
        // the initial read, then immediately re-observe World head so a commit
        // that published & retired an unowned partition before `open_funded`
        // can never silently vanish behind a freshly stamped partition.
        let subscription = self
            .runtime
            .primary_provider
            .live_delivery_ref()
            .open_funded(&execution, retained_product.observation())
            .map_err(|denial| map_open_live_admission_denial(denial, request.query.name()))?;
        let current_head = self
            .runtime
            .product_runtime
            .admit_product_branch(&opening_branch)
            .map_err(|cause| map_open_branch_admission_denial(cause, request.query.name()))?;
        if current_head.observation().lifecycle_incarnation() != opening_incarnation
            || current_head.observation().selected_commit()
                != retained_product.observation().selected_commit()
        {
            return Err(open_denial(
                WorthQueryApplicationLiveOpenDenialKind::ProviderVersionUnavailable,
                request.query.name(),
            ));
        }
        let cursor = subscription.cursor();

        let access =
            WorthQueryApplicationQueryAccessContext::new(request.principal, &request.scope);
        let query_controls = WorthQueryApplicationQueryControls::product_live(
            request.product.read_lease(),
            request.application_basis,
            request.controls.maximum_materialized_record_count(),
            request.controls.request(),
        )
        .with_original_settled_producer(request.producer);
        let (admitted_parameters, query_controls) = self
            .prepare_application_query_admission(
                &request.query,
                &access,
                request.parameters.clone(),
                query_controls,
            )
            .map_err(open_admission_denial)?;
        let mut plan = self
            .finish_application_query_admission(
                &execution,
                &request.query,
                &access,
                admitted_parameters,
                query_controls,
                request.pending_governance,
            )
            .map_err(open_admission_denial)?;
        if let Err(denial) = refresh_governed_authorization(&execution, self, &mut plan) {
            let _ = plan.basis.release();
            return Err(open_read_denial(denial, request.query.name()));
        }

        let (raw, authorization_work, read_proof) =
            match execute_authorized_read(&execution, self, &plan, |runtime, graph, plan| {
                let mut raw = read_bounded_root_rows(runtime, &execution, graph, plan, None)?;
                if QueryResult::PROJECTS_PERSISTENT_NAMES {
                    observe_persistent_names(
                        runtime,
                        &execution,
                        plan,
                        &mut raw,
                        QueryResult::PROJECTS_GROUP_NAMES,
                    );
                }
                Ok(raw)
            }) {
                Ok(ok) => ok,
                Err(denial) => {
                    let _ = plan.basis.release();
                    return Err(open_read_denial(denial, request.query.name()));
                }
            };

        let basis = admit_live_managed_basis(
            self,
            request.controls.buffer_capacity(),
            &plan.graph_work,
            request.query.name(),
            execution,
        )?;
        let completed =
            complete_one_shot(&execution, self, plan, raw, authorization_work, read_proof)
                .map_err(|denial| map_open_one_shot_denial(denial, request.query.name()))?;
        let read_completion = completed.read.receipt().read_completion().clone();
        let layout = self.runtime.primary_provider.graph_ref().layout_ref();
        let footprint = WholeResultInvalidationFootprint::capture(
            &execution,
            layout,
            request.query.name(),
            request.controls.request(),
            completed.read.result_set_observation(),
            completed.read.observed_sources(),
        )
        .map_err(|stop| map_open_footprint_stop(stop, request.query.name()))?;
        let (rows, observed_sources, result_set_observation, receipt) =
            completed.read.into_whole_result_parts();
        let snapshot = WorthQueryApplicationWholeResultLiveSnapshot::new(
            rows,
            observed_sources,
            result_set_observation,
            receipt,
        );
        Ok(WorthQueryApplicationWholeResultLiveLease {
            runtime: super::LiveRuntimeCustody::Borrowed(self),
            termination_failed: false,
            query: request.query,
            scope: request.scope,
            parameters: request.parameters,
            controls: request.controls,
            opening_branch,
            opening_incarnation,
            opening_product: Some(retained_product),
            governance: completed.governance,
            basis: super::LiveBasisSlot(Some(basis)),
            read_completion: Some(read_completion),
            subscription: Some(subscription),
            cursor,
            footprint: Some(footprint),
            initial: Some(snapshot),
            _principal: PhantomData,
            _thread_affinity: PhantomData,
        })
    }
}
