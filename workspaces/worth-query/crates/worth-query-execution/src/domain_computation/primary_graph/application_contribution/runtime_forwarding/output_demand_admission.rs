//! Thin forwarding shims for `application_contribution` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_admission {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        pub fn output_demand_resource_profile(
            &self,
        ) -> crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile
        {
            self.inner.output_demand_resource_profile()
        }
        pub fn admit_output_demand<Family>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner
                .admit_output_demand::<Family>(phase, source_result, limits)
        }
        #[doc(hidden)]
        pub(crate) fn admit_required_output_demand<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner
                .admit_required_output_demand::<Family>(execution, source_result, limits)
        }
        #[doc(hidden)]
        pub(crate) fn admit_recovered_output_demand<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            current_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
            source_receipt: &crate::domain_computation::primary_graph::WorthQueryOutputSourceEvidence,
        ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner.admit_recovered_output_demand::<Family>(
                execution,
                source_result,
                current_result,
                limits,
                source_receipt,
            )
        }
        pub fn admit_performed_output_demand<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
            prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
            retained_program_basis: std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner.admit_performed_output_demand::<Family>(
                execution,
                source_result,
                limits,
                prepared,
                retained_program_basis,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_durable_output {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub(crate) fn retain_coupled_output_records(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
            observation: &worth_runtime_world::facade::ProductBranchObservation,
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner
                .retain_coupled_output_records(execution, receipt, observation)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_selected_program {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_declaration::facade::application_program::ApplicationProgramIdentity;
    use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::WorthQueryOutputDemandAdvance;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        #[allow(clippy::too_many_arguments)]
        pub fn advance_selected_program_output_demand<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            request_scope: &WorthQueryRequestScope,
            delivery_branch: crate::basis::WorthQueryProductBranch,
            disclosure: WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            identity: ApplicationProgramIdentity,
            revision: ApplicationProgramRevision,
        ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static,
        {
            self.inner.advance_selected_program_output_demand::<Family>(
                execution,
                phase,
                access,
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
                identity,
                revision,
            )
        }
        #[allow(clippy::too_many_arguments)]
        pub fn advance_selected_program_output_demand_from_retained<Family>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            request_scope: &WorthQueryRequestScope,
            delivery_branch: crate::basis::WorthQueryProductBranch,
            identity: ApplicationProgramIdentity,
            revision: ApplicationProgramRevision,
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
            self.inner
                .advance_selected_program_output_demand_from_retained::<Family>(
                    phase,
                    access,
                    demand,
                    principal,
                    request_scope,
                    delivery_branch,
                    identity,
                    revision,
                )
        }
    }
}
