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
    pub(super) installed_product: Option<super::installed_product::WorthQueryInstalledProduct>,
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
