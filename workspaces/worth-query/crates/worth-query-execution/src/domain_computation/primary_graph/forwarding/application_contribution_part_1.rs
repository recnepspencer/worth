//! Thin forwarding shims for `application_contribution` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_durable {
    use crate::facade::product::WorthQueryProductBranch;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerOutputFamily;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::durable::WorthQueryOriginalOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::denial::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type FamilySourceValue<Schema, Family> =
    <<FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value;
    type FamilySourceQuery<Schema, Family> =
    <FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query;
    type FamilySource<Schema, Family> = <Family as WorthQueryProducerOutputFamily<Schema>>::Source;
    impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn readmit_original_output_demand<F>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
            request: &WorthQueryRequestScope,
            branch: WorthQueryProductBranch,
            original: &WorthQueryOriginalOutputDemand,
        ) -> Result<WorthQueryAdmittedOutputDemand<S, F>, WorthQueryOutputDemandDenial>
        where
            F: WorthQueryProducerOutputFamily<S>,
            FamilySourceValue<S, F>:
                crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                        S,
                        FamilySourceQuery<S, F>,
                    > + 'static,
            FamilySourceQuery<S, F>: 'static,
        {
            self.inner
                .readmit_original_output_demand::<F>(phase, principal, request, branch, original)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandAdvance;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        pub fn advance_output_demand<Family>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            request_scope: &WorthQueryRequestScope,
            delivery_branch: crate::basis::WorthQueryProductBranch,
            disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
        ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static,
        {
            self.inner.advance_output_demand::<Family>(
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
            )
        }
        pub fn advance_output_demand_from_retained<Family>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            request_scope: &WorthQueryRequestScope,
            delivery_branch: crate::basis::WorthQueryProductBranch,
        ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>:
                crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                        Schema,
                        FamilySourceQuery<Schema, Family>,
                    > + 'static,
            FamilySourceQuery<Schema, Family>: 'static,
        {
            self.inner.advance_output_demand_from_retained::<Family>(
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
            )
        }
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn advance_program_output_demand<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            request_scope: &WorthQueryRequestScope,
            delivery_branch: crate::basis::WorthQueryProductBranch,
            disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
        ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static,
        {
            self.inner.advance_program_output_demand::<Family>(
                execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
            )
        }
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn advance_history_output_demand<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            request_scope: &WorthQueryRequestScope,
            delivery_branch: crate::basis::WorthQueryProductBranch,
            history: crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation<'_>,
        ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>:
                crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                        Schema,
                        FamilySourceQuery<Schema, Family>,
                    > + 'static,
            FamilySourceQuery<Schema, Family>: 'static,
        {
            self.inner.advance_history_output_demand::<Family>(
                execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                history,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_selection {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::WorthQueryObservedSource;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::denial::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQuerySelectedApplicationProducer;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        pub fn select_output_producer<Family>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            source: &WorthQueryObservedSource<
                <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            >,
            profile_kind: &'static str,
        ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner
                .select_output_producer::<Family>(phase, source, profile_kind)
        }
    }
}
