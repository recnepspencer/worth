//! Thin forwarding shims for `application_runtime` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_recovery_custody_readmission {
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial as Denial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
    use worth_query_installation::facade::ApplicationSchema;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn mint_admitted_recovery_handle<Operation, Input, Scope, InputBinding>(&self, phase: &WorthQueryAdvancementPhase<'_>, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>, binding: WorthQueryApplicationIdempotencyBinding, input: &worth_query_declaration::facade::application_operation::ApplicationEncodedInput<
                InputBinding,
            >) -> Result<WorthQueryRecoveryHandle, Denial>
        where
            InputBinding:
                worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                    Value = Input,
                >,
        Input: Clone + serde::Serialize + Send + Sync + 'static,{
            self.inner
                .mint_admitted_recovery_handle::<Operation, Input, Scope, InputBinding>(
                    phase, admission, binding, input,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_recovery_custody_settlement {
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial as Denial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_execution::ExecutionRequest;
    use worth_query_installation::facade::ApplicationSchema;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn settle_recovery_custody(
            &self,
            execution: &ExecutionRequest<'_, '_>,
        ) -> Result<(), Denial> {
            self.inner.settle_recovery_custody(execution)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_recovery_custody_source {
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial as Denial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_runtime_world::facade::WorldProgramActivationHistory;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn admit_recovery_claim_sources(
            &self,
            originals: &WorldProgramActivationHistory,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<(), Denial> {
            self.inner
                .admit_recovery_claim_sources(originals, execution)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_cleanup {
    use std::num::NonZeroUsize;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundAdmissionDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use crate::domain_computation::application_aftermath::WorthQueryInboundCleanupReport;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        pub fn cleanup_completed_inbound_occurrences(
            &self,
            handle: &WorthQueryInboundVerifierHandle,
            maximum_work: NonZeroUsize,
        ) -> Result<WorthQueryInboundCleanupReport, Denial> {
            self.inner
                .cleanup_completed_inbound_occurrences(handle, maximum_work)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_cost {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::cost::WorthQueryInboundCostObservation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        pub fn observe_inbound_cost(
            &self,
            handle: &WorthQueryInboundVerifierHandle,
        ) -> Option<WorthQueryInboundCostObservation> {
            self.inner.observe_inbound_cost(handle)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_maintenance {
    use std::num::NonZeroUsize;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundAdmissionDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::maintenance::WorthQueryInboundMaintenanceReport;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn maintain_inbound_occurrences(
            &self,
            handle: &WorthQueryInboundVerifierHandle,
            maximum_work: NonZeroUsize,
            request: &WorthQueryRequestScope,
        ) -> Result<WorthQueryInboundMaintenanceReport, Denial> {
            self.inner
                .maintain_inbound_occurrences(handle, maximum_work, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_observe {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::observe::WorthQueryInboundTerminalObservation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        pub fn observe_inbound_terminal(
            &self,
            handle: &WorthQueryInboundVerifierHandle,
            correlation_token: [u8; 32],
        ) -> Option<WorthQueryInboundTerminalObservation> {
            self.inner
                .observe_inbound_terminal(handle, correlation_token)
        }
        pub fn observe_inbound_recovery_denial(&self, handle: &WorthQueryInboundVerifierHandle, correlation_token: [u8; 32]) -> Option<std::sync::Arc<worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::WorthQueryRecordedCompletionDenial>>{
            self.inner
                .observe_inbound_recovery_denial(handle, correlation_token)
        }
    }
}
