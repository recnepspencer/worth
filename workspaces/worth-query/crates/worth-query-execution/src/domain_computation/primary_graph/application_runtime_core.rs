//! State owned once by an application runtime, independent of its application types.
use super::*;

pub struct WorthQueryApplicationRuntimeCore {
    pub(super) execution: Option<WorthQueryExecutionRuntime>,
    pub(in crate::domain_computation::primary_graph) application_readiness_schema_token: String,
    pub(super) publication: WorthQueryPrimaryGraphPublication,
    pub(super) checkpoint_restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
    pub(in crate::domain_computation) authorization: Arc<WorthQueryInstalledAuthorizationRegistry>,
    pub(in crate::domain_computation) authorization_clock: Arc<WorthQueryRuntimeClock>,
    pub(in crate::domain_computation::primary_graph) relational_branch_identity:
        worth_relational::facade::branch::RelationalBranchIdentity,
    pub(crate) bridge: super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
    pub(crate) product_runtime:
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    pub(in crate::domain_computation::primary_graph) recovered_program_source_delivery:
        super::super::provider::recovered_program_source_delivery::RecoveredProgramSourceDeliveryRegistration,
    pub(in crate::domain_computation::primary_graph) basis_leases:
        super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistry,
    pub(in crate::domain_computation::primary_graph) granular_invalidation: super::super::WorthQueryGranularInvalidationInstallation,
    pub(crate) primary_provider: std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    pub(in crate::domain_computation::primary_graph) primary_graph_authority:
        Option<worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority>,

    pub(in crate::domain_computation::primary_graph) source_meanings:
        Option<super::super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry>,
    pub(in crate::domain_computation::primary_graph) next_external_dispatch_attempt: AtomicU64,
    pub(in crate::domain_computation::primary_graph) external_effect_transport:
        std::sync::OnceLock<std::sync::Arc<dyn WorthQueryExternalEffectTransport>>,
    pub(super) inbound_verifiers: std::sync::Mutex<
        std::collections::BTreeMap<
            String,
            std::sync::Arc<inbound_occurrence::WorthQueryInstalledInboundVerifier>,
        >,
    >,
    pub(super) inbound_custody: Arc<std::sync::Mutex<
        crate::domain_computation::application_aftermath::WorthQueryInboundCustody,
    >>,
    pub(super) transport_completion_custody: std::sync::Mutex<
        inbound_publication::InstalledTransportCompletionCustody,
    >,
    pub(super) inbound_maintenance_turn: AtomicU64,
    pub(crate) recovery_handles: Arc<WorthQueryRecoveryHandleRegistry>,
    pub(in crate::domain_computation::primary_graph) output_producer_routes:
        super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes,
    pub(in crate::domain_computation::primary_graph) output_readiness_routes:
        super::super::application_contribution::WorthQueryInstalledOutputReadinessRoutes,
    pub(in crate::domain_computation::primary_graph) next_output_producer_attempt: AtomicU64,
    pub(in crate::domain_computation::primary_graph) next_application_mutation_partition: AtomicU32,
    pub(in crate::domain_computation::primary_graph) output_demands: super::super::application_output_demand::WorthQueryOutputDemandRegistry,
    pub(in crate::domain_computation::primary_graph) recovered_outputs: super::super::application_output_demand::WorthQueryRecoveredOutputs,
    pub(in crate::domain_computation::primary_graph) workflow_coverage:
        crate::domain_computation::primary_graph::application_attempt::workflow::adoption::WorkflowVocabularyCoverageRegistry,
    pub(in crate::domain_computation::primary_graph) installed_operations: Arc<worth_query_installation::facade::InstalledOperationCatalog>,
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation::primary_graph) fn primary_graph_authority(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority {
        self.primary_graph_authority
            .as_ref()
            .expect("a live application owns its graph authority")
    }
    pub(in crate::domain_computation::primary_graph) fn source_meanings(
        &self,
    ) -> &super::super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry{
        self.source_meanings
            .as_ref()
            .expect("a live application owns its source meanings")
    }

    pub(in crate::domain_computation) fn execution_mut(
        &mut self,
    ) -> &mut WorthQueryExecutionRuntime {
        self.execution
            .as_mut()
            .expect("a live application owns its execution runtime")
    }
    pub(in crate::domain_computation) fn execution(&self) -> &WorthQueryExecutionRuntime {
        self.execution
            .as_ref()
            .expect("a live application owns its execution runtime")
    }
    pub(in crate::domain_computation) fn authority_identity(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.execution().authority_identity()
    }
    pub(in crate::domain_computation) fn primary_graph(
        &self,
    ) -> Option<&super::super::WorthQueryPrimaryGraph> {
        self.execution().primary_graph()
    }
    pub(in crate::domain_computation) fn installed_packages(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledPackageIndex {
        self.execution().installed_packages()
    }
    pub(in crate::domain_computation) fn retain_installed_packages(
        &self,
    ) -> Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex> {
        self.execution().retain_installed_packages()
    }
    pub(in crate::domain_computation) fn retain_primary_graph_integration_handle(
        &self,
    ) -> Option<super::super::WorthQueryPrimaryGraphIntegrationHandle> {
        self.execution().retain_primary_graph_integration_handle()
    }

    pub(in crate::domain_computation) fn output_demand_resource_profile(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile {
        self.execution().output_demand_resource_profile()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn authentication_is_expired(
        &self,
        valid_until: std::time::Instant,
    ) -> bool {
        self.authorization_clock.admission_clock.now() >= valid_until
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn publication(&self) -> &WorthQueryPrimaryGraphPublication {
        &self.publication
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn checkpoint_restore_work(
        &self,
    ) -> Option<worth_relational::facade::durability::CheckpointRestoreWork> {
        self.checkpoint_restore_work
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn granular_invalidation_installation(
        &self,
    ) -> super::super::WorthQueryGranularInvalidationInstallation {
        self.granular_invalidation.current()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn application_query_basis_observer(
        &self,
    ) -> super::super::application_query::WorthQueryApplicationBasisObserver {
        self.basis_leases.observer()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn capability_plan_compilation_evidence(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryCapabilityPlanCompilationEvidence {
        self.authorization.capability_compilation()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn workflow_compilation_reuse_counters(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::workflow::definition::WorthQueryWorkflowCompilationReuseCounters {
        self.retain_primary_graph_integration_handle()
            .expect("a published application runtime retains its primary graph")
            .workflow_compilation_reuse_counters()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn workflow_instance_progress_counters(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::workflow::WorthQueryWorkflowInstanceProgressCounters {
        self.retain_primary_graph_integration_handle()
            .expect("a published application runtime retains its primary graph")
            .workflow_instance_progress_counters()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn graph_work_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.primary_provider.application_resource_support()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn graph_work_resource_support_ref(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
{
        self.primary_provider.application_resource_support_ref()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn graph_work_provider_identity(&self) -> &str {
        self.primary_graph_authority().provider_identity()
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn close_live_delivery(
        &self,
    ) -> WorthQueryApplicationLiveDeliveryCloseReceipt {
        WorthQueryApplicationLiveDeliveryCloseReceipt {
            remaining_live_consumers: self.primary_provider.live_delivery.close(),
        }
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation) fn installed_operations(
        &self,
    ) -> &worth_query_installation::facade::InstalledOperationCatalog {
        &self.installed_operations
    }
}

#[path = "application_runtime_core_court_doors_for_test.rs"]
mod court_doors_for_test;
