//! Thin forwarding shims for `application_contribution` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_admission_restoration {
    use crate::equality_type_id::EqualityTypeId as TypeId;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::admission::WorthQueryObservedSource;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::admission::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::admission::restoration::ReadmittedOutput;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn readmit_checkpoint_output<Query>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            producer: &str,
            output_binding: TypeId,
            expected_idempotency_key: [u8; 32],
            observed_source: &WorthQueryObservedSource<Query>,
            source_epoch: crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch,
            source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
            owner_retained_program_basis: bool,
        ) -> Result<Option<ReadmittedOutput>, WorthQueryOutputDemandDenial> {
            self.inner.readmit_checkpoint_output::<Query>(
                execution,
                producer,
                output_binding,
                expected_idempotency_key,
                observed_source,
                source_epoch,
                source_scope,
                owner_retained_program_basis,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_admission_retained_observation {
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
    use crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits;
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
        pub fn admit_output_demand_at<Family>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            source_result: WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            limits: WorthQueryOutputDemandLimits,
            observation: &WorthQueryApplicationReadObservation,
        ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner
                .admit_output_demand_at::<Family>(phase, source_result, limits, observation)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_admission_source_custody {
    use crate::equality_type_id::EqualityTypeId as TypeId;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn retain_required_output_source_carrier(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
            change: std::sync::Arc<crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange>,
            observation: worth_runtime_world::facade::ProductBranchObservation,
            preparation: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRequiredOutputSourcePreparation,
            root_kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
            discovery: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
        ) -> Result<
            (
                crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
                std::sync::Arc<
                    crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
                >,
            ),
            WorthQueryOutputDemandDenial,
        > {
            self.inner.retain_required_output_source_carrier(
                execution,
                receipt,
                change,
                observation,
                preparation,
                root_kind,
                discovery,
            )
        }
        pub(crate) fn recover_discovered_output_source<Discovery>(
            &self,
            receipt: &crate::domain_computation::primary_graph::WorthQueryOutputSourceEvidence,
            root_type: TypeId,
        ) -> Result<
            (
                crate::domain_computation::primary_graph::WorthQueryOutputDiscoveryRecipe<
                    Discovery,
                >,
                std::sync::Arc<
                    crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
                >,
                crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
            ),
            WorthQueryOutputDemandDenial,
        >
        where
            Discovery: Clone + Send + Sync + 'static,
        {
            self.inner
                .recover_discovered_output_source::<Discovery>(receipt, root_type)
        }
        pub(crate) fn bind_prepared_output_source<Query, Value>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
            source: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                Query,
                Value,
            >,
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner
                .bind_prepared_output_source::<Query, Value>(phase, prepared, source)
        }
        pub(crate) fn bind_prepared_output_sources<Query, Value>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
            sources: &[crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                Query,
                Value,
            >],
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner
                .bind_prepared_output_sources::<Query, Value>(phase, prepared, sources)
        }
        pub fn discard_prepared_required_output_source(
            &self,
            prepared: crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        ) {
            self.inner.discard_prepared_required_output_source(prepared)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_request_execution_advancement_installed_opening {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::request_execution::advancement::WorthQueryAdvancementDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::request_execution::advancement::WorthQueryAdvancementPhase;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::request_execution::advancement::WorthQueryBootstrapAdvancementPhase;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::request_execution::advancement::WorthQueryForeignAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn with_host_advancement<R>(
            &self,
            body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
        ) -> Result<R, WorthQueryAdvancementDenial> {
            self.inner.with_host_advancement::<R>(body)
        }
        pub fn with_funded_host_advancement<R>(
            &self,
            allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
            body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
        ) -> Result<R, WorthQueryAdvancementDenial> {
            self.inner
                .with_funded_host_advancement::<R>(allocation_policy, body)
        }
        pub fn with_execution_request_advancement<R>(
            &self,
            execution: worth_execution::ExecutionRequest<'_, '_>,
            body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
        ) -> Result<R, WorthQueryAdvancementDenial> {
            self.inner
                .with_execution_request_advancement::<R>(execution, body)
        }
        pub fn with_scoped_execution_request_advancement<R>(
            &self,
            execution: worth_execution::ExecutionRequest<'_, '_>,
            policy: worth_foundational::ExecutionRequestPolicy,
            request: &WorthQueryRequestScope,
            body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
        ) -> Result<R, WorthQueryAdvancementDenial> {
            self.inner
                .with_scoped_execution_request_advancement::<R>(execution, policy, request, body)
        }
        pub fn with_bootstrap_handed_off_advancement<R>(
            &self,
            bootstrap: &WorthQueryBootstrapAdvancementPhase<'_>,
            retain_returned_outcome: bool,
            body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
        ) -> Result<R, WorthQueryAdvancementDenial> {
            self.inner.with_bootstrap_handed_off_advancement::<R>(
                bootstrap,
                retain_returned_outcome,
                body,
            )
        }
        pub fn with_funded_application_advancement<R>(
            &self,
            request: &WorthQueryRequestScope,
            allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
            body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
        ) -> Result<R, WorthQueryAdvancementDenial> {
            self.inner
                .with_funded_application_advancement::<R>(request, allocation_policy, body)
        }
        pub fn with_funded_application_completion<R>(
            &self,
            request: &WorthQueryRequestScope,
            allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
            body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
        ) -> Result<R, WorthQueryAdvancementDenial> {
            self.inner
                .with_funded_application_completion::<R>(request, allocation_policy, body)
        }
        pub fn validate_application_advancement(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
        ) -> Result<(), WorthQueryForeignAdvancementPhase> {
            self.inner.validate_application_advancement(phase)
        }
        pub fn with_application_advancement<R>(
            &self,
            request: &WorthQueryRequestScope,
            body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
        ) -> Result<R, WorthQueryAdvancementDenial> {
            self.inner.with_application_advancement::<R>(request, body)
        }
    }
}
