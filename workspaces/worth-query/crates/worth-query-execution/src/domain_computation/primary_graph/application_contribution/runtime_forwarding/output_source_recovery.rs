//! Thin forwarding shims for `application_contribution` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_source_recovery {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
    use crate::domain_computation::primary_graph::WorthQueryOutputSourceEvidence;
    use crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource;
    use crate::equality_type_id::EqualityTypeId as TypeId;
    use worth_query_installation::facade::ApplicationSchema;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn resolve_output_source_replay(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            selector: crate::domain_computation::primary_graph::WorthQueryOutputSourceReplay<'_>,
            kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
            root_identity: &str,
        ) -> Result<WorthQueryOutputSourceEvidence, WorthQueryOutputDemandDenial> {
            self.inner
                .resolve_output_source_replay(phase, selector, kind, root_identity)
        }
        pub(crate) fn validate_recovered_output_root_kind(
            &self,
            receipt: &WorthQueryOutputSourceEvidence,
            kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner
                .validate_recovered_output_root_kind(receipt, kind)
        }
        pub(crate) fn complete_prepared_output_source(
            &self,
            receipt: &WorthQueryOutputSourceEvidence,
        ) {
            self.inner.complete_prepared_output_source(receipt)
        }
        pub(crate) fn recover_prepared_output_source(
            &self,
            receipt: &WorthQueryOutputSourceEvidence,
            root_type: TypeId,
        ) -> Option<(
            std::sync::Arc<WorthQueryApplicationReadObservation>,
            WorthQueryPreparedRequiredOutputSource,
        )> {
            self.inner
                .recover_prepared_output_source(receipt, root_type)
        }
        pub(crate) fn ensure_recovered_output_source_bound<Query, Value>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            prepared: &WorthQueryPreparedRequiredOutputSource,
            source: &WorthQueryApplicationOutputDemandSource<Query, Value>,
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner
                .ensure_recovered_output_source_bound::<Query, Value>(phase, prepared, source)
        }
        pub(crate) fn validate_recovered_output_source_currentness<Query, Value>(
            &self,
            prepared: &WorthQueryPreparedRequiredOutputSource,
            retained: &WorthQueryApplicationOutputDemandSource<Query, Value>,
            current: &WorthQueryApplicationOutputDemandSource<Query, Value>,
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner
                .validate_recovered_output_source_currentness::<Query, Value>(
                    prepared, retained, current,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_unavailable {
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn finish_unavailable_output_demand<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            prepared: &WorthQueryPreparedRequiredOutputSource,
            principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            scope: &WorthQueryRequestScope,
            branch: crate::basis::WorthQueryProductBranch,
            disclosure: WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
        ) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner.finish_unavailable_output_demand::<Family>(
                execution, phase, demand, prepared, principal, scope, branch, disclosure,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_admission_dependent {
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
        pub(crate) fn admit_dependent_output_demand<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
            retained_basis: std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner.admit_dependent_output_demand::<Family>(
                execution,
                source_result,
                limits,
                retained_basis,
            )
        }
    }
}
