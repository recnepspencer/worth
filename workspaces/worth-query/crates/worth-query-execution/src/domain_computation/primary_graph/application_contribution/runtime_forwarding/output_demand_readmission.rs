//! Thin forwarding shims for `application_output_demand` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_completed_outputs {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::WorthQueryFulfilledRequiredRoot;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
    use crate::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial;
    use crate::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::completed_outputs::WorthQueryCompletedRequiredOutputs;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
        #[allow(clippy::too_many_arguments)]
        pub fn qualify_completed_required_outputs<'s>(
            self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            scope: &WorthQueryRequestScope,
            source: &WorthQueryApplicationCommitReceipt,
            fulfilled: &WorthQueryFulfilledRequiredRoot,
            root: &'s WorthQueryOutputDemandSettlement,
            dependents: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
            dependencies: Option<&WorthQueryRetainedQueryDependencies>,
        ) -> Result<WorthQueryCompletedRequiredOutputs, WorthQueryRequiredSetCurrentnessDenial>
        {
            self.inner.qualify_completed_required_outputs(
                phase,
                access,
                scope,
                source,
                fulfilled,
                root,
                dependents,
                dependencies,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_completed_read {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
    use crate::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial;
    use crate::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies;
    use std::sync::Arc;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::completed_read::WorthQueryCompletedReadOutputs;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
        #[allow(clippy::too_many_arguments)]
        pub fn qualify_completed_read_outputs<'s>(
            self,
            phase: &WorthQueryAdvancementPhase<'_>,
            access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            scope: &WorthQueryRequestScope,
            source: &Arc<WorthQueryApplicationReadObservation>,
            root: &'s WorthQueryOutputDemandSettlement,
            dependents: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
            dependencies: Option<&WorthQueryRetainedQueryDependencies>,
        ) -> Result<WorthQueryCompletedReadOutputs, WorthQueryRequiredSetCurrentnessDenial>
        {
            self.inner.qualify_completed_read_outputs(
                phase,
                access,
                scope,
                source,
                root,
                dependents,
                dependencies,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_completed_set {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
    use crate::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial;
    use crate::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies;
    use std::sync::Arc;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn qualify_completed_output_set<'s>(
            self,
            phase: &WorthQueryAdvancementPhase<'_>,
            access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            scope: &WorthQueryRequestScope,
            root: &'s WorthQueryOutputDemandSettlement,
            dependents: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
            dependencies: Option<&WorthQueryRetainedQueryDependencies>,
            retained_bytes: u64,
        ) -> Result<
            (
                Arc<WorthQueryApplicationReadObservation>,
                Option<worth_execution::ExecutionMemoryReservation>,
            ),
            WorthQueryRequiredSetCurrentnessDenial,
        > {
            self.inner.qualify_completed_output_set(
                phase,
                access,
                scope,
                root,
                dependents,
                dependencies,
                retained_bytes,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_currentness {
    use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
    use worth_query_installation::facade::ApplicationSchema;
    impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
        pub fn require_current_output_settlements<'settlement>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            settlements: impl IntoIterator<Item = &'settlement WorthQueryOutputDemandSettlement>,
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner
                .require_current_output_settlements(phase, settlements)
        }
        pub fn require_current_output_receipts<'receipt>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            receipts: impl IntoIterator<Item = &'receipt WorthQueryApplicationCommitReceipt>,
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner.require_current_output_receipts(phase, receipts)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_settlement_runtime_selection {
    use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        pub fn select_output_demand_settlement(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            settlement: &WorthQueryOutputDemandSettlement,
        ) -> Result<WorthQuerySelectedProductOperation<'_, Schema>, WorthQueryOutputDemandDenial>
        {
            self.inner
                .select_output_demand_settlement(execution, settlement)
                .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_discovery_recipe_read {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use crate::domain_computation::primary_graph::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial as Denial;
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryScopeResolution;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::discovery_recipe::WorthQueryOutputDiscoveryRecipe;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn read_recorded_output_discovery<B>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            recipe: &WorthQueryOutputDiscoveryRecipe<B::Input>,
            external: &WorthQueryAuthenticatedExternalPrincipal<S>,
            request: &WorthQueryRequestScope,
            original: &WorthQueryApplicationReadObservation,
        ) -> Result<
            WorthQueryApplicationOutputDemandSource<
                B::Query,
                <B::ResultBinding as ApplicationStructuredValueBinding>::Value,
            >,
            Denial,
        >
        where
            B: ApplicationQueryBinding<S>,
            B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
            <B::ResultBinding as ApplicationStructuredValueBinding>::Value:
                WorthQueryApplicationProjection<S, B::Query>,
        {
            self.inner.read_recorded_output_discovery::<B>(
                phase, __arg1, recipe, external, request, original,
            )
        }
    }
}
