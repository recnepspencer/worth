//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryRuntimeBuilder>(
            "workspaces/worth-query/crates/worth-query/src/runtime/builder.rs",
            "WorthQueryRuntimeBuilder",
            128 * 1024,
        );
        owner.field::<worth_query_execution::facade::integration::WorthQueryProductWorldResources>(
            "product_world_resources",
        );
        owner.field::<Option<Result<Box<dyn WorthQueryRuntimeBackend>, WorthQueryRuntimeError>>>(
            "backend",
        );
        owner.field::<WorthQueryRuntimeBackendParts>("backend_parts");
        owner.field::<QueuedInvariantRegistrations>("queued_invariant_registrations");
        owner.field::<crate::domain_installation::WorthQueryPendingDomainInstallations>(
            "pending_domain_installations",
        );
        owner.field::<crate::domain_installation::WorthQueryPendingGraphParticipations>(
            "pending_graph_participations",
        );
        owner.field::<crate::domain_installation::WorthQueryPendingDomainOperationExecutors>(
            "pending_domain_operation_executors",
        );
        owner.field::<crate::domain_installation::WorthQueryPendingWorkflowStageExecutors>(
            "pending_workflow_stage_executors",
        );
        owner.field::<crate::domain_installation::WorthQueryPendingWorkflowParallelAdmissionProviders>("pending_workflow_parallel_admission_providers");
        owner.field::<[Option<crate::domain_installation::WorthQueryConsumerSupportPosture>;
            crate::domain_installation::WorthQueryConsumerSupportDimension::COUNT]>(
            "consumer_support_postures",
        );
        owner.field::<crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry>("native_aspect_contracts");
        owner.field::<Option<worth_runtime_bridge::facade::RuntimeBridge>>(
            "conditional_runtime_bridge",
        );
        owner.field::<Option<Box<worth_signal::facade::SignalGraph>>>("conditional_signal_graph");
        owner.field::<Option<super::super::WorthQueryConditionalExecutionResources>>(
            "conditional_execution_resources",
        );
        owner.field::<Option<product_bridge::WorthQueryPendingRelationalProductBridge>>(
            "pending_relational_product_bridge",
        );
        owner.field::<Vec<Box<dyn crate::domain_installation::PendingConditionalInstallation>>>(
            "pending_conditional_installations",
        );
        owner.field::<Vec<super::super::WorthQueryOwnedAsyncRequestDeclaration>>(
            "pending_owned_async_declarations",
        );
        owner.field::<Option<Box<dyn primary_graph::PendingPrimaryGraphInstallation>>>(
            "pending_primary_graph_installation",
        );
        owner.field::<Option<
        worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    >>("primary_runtime_invalidation_installation");
        owner.field::<Option<worth_query_execution::facade::runtime::WorthQueryExecutionRuntimeInstallation>>("host_execution_installation");
    }
}
