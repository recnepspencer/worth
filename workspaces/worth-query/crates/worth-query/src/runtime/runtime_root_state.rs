use std::collections::{BTreeMap, BTreeSet};

use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::program::{WorthQueryProgram, WorthQueryProgramTrace};
use crate::session_label::WorthQuerySessionLabel;
use crate::subscription::ActiveSubscriptionRuntime;

use super::backend::WorthQueryRuntimeBackend;
use super::computed::{WorthQueryComputedDependencyIndex, WorthQueryDerivedViewRuntime};
use super::delivery::WorthQueryRuntimeLiveSubscriptionState;
use super::effect::{WorthQueryEffectIndex, WorthQueryEffectRuntime, WorthQueryEffectTarget};
use super::installed_live_routing::WorthQueryInstalledLiveRoutes;
use super::managed_live_resource::WorthQueryManagedLiveWorkspaceCapability;
use super::native_aspect_contracts::WorthQueryNativeAspectContractRegistry;
use super::shared_projection_owners::WorthQuerySharedProjectionOwnerRegistry;
use super::shared_read_pins::WorthQuerySharedReadPinRegistry;
use super::support::WorthQueryRuntimeEvidenceAuthority;
use super::surface::{
    WorthQueryDerivedMaterializationTarget, WorthQueryLiveArtifactTarget,
    WorthQueryProgramInstallationIdentity, WorthQueryProgramRunIdentity,
};
use super::WorthQueryRuntimeAuthorityIdentity;

pub struct WorthQueryRuntime {
    pub(super) backend: Box<dyn WorthQueryRuntimeBackend>,
    pub(super) evidence_authority: WorthQueryRuntimeEvidenceAuthority,
    pub(super) authority_identity: WorthQueryRuntimeAuthorityIdentity,
    pub(super) execution_runtime:
        worth_query_execution::facade::runtime::WorthQueryExecutionRuntime,
    pub(super) execution_installation_authority:
        worth_query_execution::facade::runtime::WorthQueryExecutionInstallationAuthority,
    pub(super) primary_graph_publication:
        Option<worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphPublication>,
    pub(super) primary_runtime_invalidation_installation: Option<
        worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    >,
    pub(super) domain_installation_registry:
        crate::domain_installation::WorthQueryDomainInstallationRegistry,
    pub(super) domain_operation_executor_registry:
        crate::domain_installation::WorthQueryDomainOperationExecutorRegistry,
    pub(super) workflow_stage_executor_registry:
        crate::domain_installation::WorthQueryWorkflowStageExecutorRegistry,
    pub(super) workflow_parallel_admission_provider_registry:
        crate::domain_installation::WorthQueryWorkflowParallelAdmissionProviderRegistry,
    pub(super) graph_participation_registry:
        crate::domain_installation::WorthQueryInstalledGraphParticipationRegistry,
    pub(super) installed_product: super::installed_product::WorthQueryInstalledProduct,
    pub(super) conditional_execution_registry:
        crate::domain_installation::WorthQueryConditionalExecutionRegistry,
    pub(super) installed_owned_async_declarations:
        BTreeMap<String, super::WorthQueryInstalledOwnedAsyncDeclaration>,
    pub(super) installed_live_routes: WorthQueryInstalledLiveRoutes,
    pub(super) shared_projection_owners: WorthQuerySharedProjectionOwnerRegistry,
    pub(super) consumer_support_profile:
        crate::domain_installation::WorthQueryConsumerSupportProfile,
    pub(super) native_aspect_contracts: WorthQueryNativeAspectContractRegistry,
    pub(super) preview_session_labels: BTreeSet<WorthQuerySessionLabel>,
    pub(super) branch_session_labels: BTreeSet<WorthQuerySessionLabel>,
    pub(super) active_subscriptions: ActiveSubscriptionRuntime,
    pub(super) live_subscriptions:
        BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryRuntimeLiveSubscriptionState>,
    pub(super) granular_projection_states:
        BTreeMap<String, crate::live::WorthQueryProjectionMaintenanceState>,
    pub(super) materialized_read_views:
        BTreeMap<WorthQueryLiveArtifactTarget, DeclarativeLiveQueryRequest>,
    pub(super) live_subscription_index:
        super::live_subscription_target_index::WorthQueryLiveSubscriptionTargetIndex,
    pub(super) installed_programs:
        BTreeMap<WorthQueryProgramInstallationIdentity, WorthQueryProgram>,
    pub(super) run_traces: BTreeMap<WorthQueryProgramRunIdentity, WorthQueryProgramTrace>,
    pub(super) derived_views:
        BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
    pub(super) shared_read_pins: WorthQuerySharedReadPinRegistry,
    pub(super) published_artifacts: super::published_artifacts::WorthQueryPublishedArtifactRegistry,
    pub(super) journal_replay: super::journal_replay::WorthQueryJournalReplayRegistry,
    pub(super) derived_dependency_index: WorthQueryComputedDependencyIndex,
    pub(super) effects: BTreeMap<WorthQueryEffectTarget, WorthQueryEffectRuntime>,
    pub(super) effect_index: WorthQueryEffectIndex,
    pub(super) managed_live_resource_capability:
        std::sync::Arc<WorthQueryManagedLiveWorkspaceCapability>,
    pub(super) next_run_id: u64,
}

pub(super) struct WorthQueryRoutedMutationSummary {
    pub(super) affected_live_view_targets: Vec<WorthQueryLiveArtifactTarget>,
    pub(super) affected_derived_view_targets: Vec<WorthQueryDerivedMaterializationTarget>,
    pub(super) considered_computed_view_count: usize,
    pub(super) considered_effect_count: usize,
    pub(super) delivered_effect_count: usize,
    pub(super) pending_write_intent_count: usize,
    pub(super) suppressed_effect_count: usize,
    pub(super) meaningful_effect_suppression_count: usize,
    pub(super) effect_expression_failure_count: usize,
    pub(super) refresh_fallback: bool,
}
#[cfg(test)]
#[path = "runtime_root_state_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
fn collect_owned_inline_type_sizes(census: &mut crate::inline_type_sizes::Census) {
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
        owner.field::<super::installed_product::WorthQueryInstalledProduct>("installed_product");
        owner.field::<crate::domain_installation::WorthQueryConditionalExecutionRegistry>(
            "conditional_execution_registry",
        );
        owner.field::<BTreeMap<String, super::WorthQueryInstalledOwnedAsyncDeclaration>>(
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
        owner
            .field::<super::live_subscription_target_index::WorthQueryLiveSubscriptionTargetIndex>(
                "live_subscription_index",
            );
        owner.field::<BTreeMap<WorthQueryProgramInstallationIdentity, WorthQueryProgram>>(
            "installed_programs",
        );
        owner.field::<BTreeMap<WorthQueryProgramRunIdentity, WorthQueryProgramTrace>>("run_traces");
        owner.field::<BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>>("derived_views");
        owner.field::<WorthQuerySharedReadPinRegistry>("shared_read_pins");
        owner.field::<super::published_artifacts::WorthQueryPublishedArtifactRegistry>(
            "published_artifacts",
        );
        owner.field::<super::journal_replay::WorthQueryJournalReplayRegistry>("journal_replay");
        owner.field::<WorthQueryComputedDependencyIndex>("derived_dependency_index");
        owner.field::<BTreeMap<WorthQueryEffectTarget, WorthQueryEffectRuntime>>("effects");
        owner.field::<WorthQueryEffectIndex>("effect_index");
        owner.field::<std::sync::Arc<WorthQueryManagedLiveWorkspaceCapability>>(
            "managed_live_resource_capability",
        );
        owner.field::<u64>("next_run_id");
    }
}
