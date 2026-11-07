//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryRuntime>(
            "workspaces/worth-query/crates/worth-query/src/runtime/runtime_root_state.rs",
            "WorthQueryRuntime",
            128 * 1024,
        );
        owner.field::<Box<dyn WorthQueryRuntimeBackend>>("backend");
        owner.field::<WorthQueryRuntimeEvidenceAuthority>("evidence_authority");
        owner.field::<WorthQueryRuntimeAuthorityIdentity>("authority_identity");
        owner.field::<worth_query_execution::facade::runtime::WorthQueryExecutionRuntime>(
            "execution_runtime",
        );
        owner.field::<worth_query_execution::facade::runtime::WorthQueryExecutionInstallationAuthority>("execution_installation_authority");
        owner.field::<Option<
            worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphPublication,
        >>("primary_graph_publication");
        owner.field::<Option<
        worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    >>("primary_runtime_invalidation_installation");
        owner.field::<crate::domain_installation::WorthQueryDomainInstallationRegistry>(
            "domain_installation_registry",
        );
        owner.field::<crate::domain_installation::WorthQueryDomainOperationExecutorRegistry>(
            "domain_operation_executor_registry",
        );
        owner.field::<crate::domain_installation::WorthQueryWorkflowStageExecutorRegistry>(
            "workflow_stage_executor_registry",
        );
        owner.field::<crate::domain_installation::WorthQueryWorkflowParallelAdmissionProviderRegistry>("workflow_parallel_admission_provider_registry");
        owner.field::<crate::domain_installation::WorthQueryInstalledGraphParticipationRegistry>(
            "graph_participation_registry",
        );
        owner.field::<super::super::installed_product::WorthQueryInstalledProduct>(
            "installed_product",
        );
        owner.field::<crate::domain_installation::WorthQueryConditionalExecutionRegistry>(
            "conditional_execution_registry",
        );
        owner.field::<BTreeMap<String, super::super::WorthQueryInstalledOwnedAsyncDeclaration>>(
            "installed_owned_async_declarations",
        );
        owner.field::<WorthQueryInstalledLiveRoutes>("installed_live_routes");
        owner.field::<WorthQuerySharedProjectionOwnerRegistry>("shared_projection_owners");
        owner.field::<crate::domain_installation::WorthQueryConsumerSupportProfile>(
            "consumer_support_profile",
        );
        owner.field::<WorthQueryNativeAspectContractRegistry>("native_aspect_contracts");
        owner.field::<BTreeSet<WorthQuerySessionLabel>>("preview_session_labels");
        owner.field::<BTreeSet<WorthQuerySessionLabel>>("branch_session_labels");
        owner.field::<ActiveSubscriptionRuntime>("active_subscriptions");
        owner.field::<BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryRuntimeLiveSubscriptionState>>("live_subscriptions");
        owner.field::<BTreeMap<String, crate::live::WorthQueryProjectionMaintenanceState>>(
            "granular_projection_states",
        );
        owner.field::<BTreeMap<WorthQueryLiveArtifactTarget, DeclarativeLiveQueryRequest>>(
            "materialized_read_views",
        );
        owner.field::<super::super::live_subscription_target_index::WorthQueryLiveSubscriptionTargetIndex>("live_subscription_index");
        owner.field::<BTreeMap<WorthQueryProgramInstallationIdentity, WorthQueryProgram>>(
            "installed_programs",
        );
        owner.field::<BTreeMap<WorthQueryProgramRunIdentity, WorthQueryProgramTrace>>("run_traces");
        owner.field::<BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>>("derived_views");
        owner.field::<WorthQuerySharedReadPinRegistry>("shared_read_pins");
        owner.field::<super::super::published_artifacts::WorthQueryPublishedArtifactRegistry>(
            "published_artifacts",
        );
        owner.field::<super::super::journal_replay::WorthQueryJournalReplayRegistry>(
            "journal_replay",
        );
        owner.field::<WorthQueryComputedDependencyIndex>("derived_dependency_index");
        owner.field::<BTreeMap<WorthQueryEffectTarget, WorthQueryEffectRuntime>>("effects");
        owner.field::<WorthQueryEffectIndex>("effect_index");
        owner.field::<std::sync::Arc<WorthQueryManagedLiveWorkspaceCapability>>(
            "managed_live_resource_capability",
        );
        owner.field::<u64>("next_run_id");
    }
}
