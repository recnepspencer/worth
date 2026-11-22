//! Crate-boundary internal port for `WorthQueryApplicationRuntimeCore` backing
//! fields. Never exported on `worth-query-execution::facade` or Host.

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use super::super::*;
use super::WorthQueryApplicationRuntimeCore;

#[path = "application_runtime_core_internal_port_impl.rs"]
mod impls;

pub trait ApplicationRuntimeCoreInternalPort: Sized {
    #[allow(clippy::too_many_arguments)]
    fn from_parts(
        execution: Option<WorthQueryExecutionRuntime>,
        application_readiness_schema_token: String,
        publication: WorthQueryPrimaryGraphPublication,
        checkpoint_restore_work: Option<
            worth_relational::facade::durability::CheckpointRestoreWork,
        >,
        authorization: Arc<WorthQueryInstalledAuthorizationRegistry>,
        authorization_clock: Arc<WorthQueryRuntimeClock>,
        relational_branch_identity: worth_relational::facade::branch::RelationalBranchIdentity,
        bridge: super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
        product_runtime: crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
        recovered_program_source_delivery: super::super::super::provider::recovered_program_source_delivery::RecoveredProgramSourceDeliveryRegistration,
        basis_leases: super::super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistry,
        granular_invalidation: super::super::super::WorthQueryGranularInvalidationInstallation,
        primary_provider: super::super::super::provider::PrimaryProviderCustody,
        primary_graph_authority: Option<home_retirement::PrimaryGraphParticipationCustody>,
        source_meanings: Option<super::super::super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry>,
        next_external_dispatch_attempt: AtomicU64,
        external_effect_transport: std::sync::OnceLock<Arc<dyn WorthQueryExternalEffectTransport>>,
        inbound_verifiers: std::sync::Mutex<
            std::collections::BTreeMap<
                String,
                Arc<inbound_occurrence::WorthQueryInstalledInboundVerifier>,
            >,
        >,
        inbound_custody: Arc<
            std::sync::Mutex<
                crate::domain_computation::application_aftermath::WorthQueryInboundCustody,
            >,
        >,
        transport_completion_custody: std::sync::Mutex<
            inbound_publication::InstalledTransportCompletionCustody,
        >,
        inbound_maintenance_turn: AtomicU64,
        dispatch_journal: Arc<
            crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal,
        >,
        recovery_handles: Arc<WorthQueryRecoveryHandleRegistry>,
        output_producer_routes: super::super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes,
        output_readiness_routes: super::super::super::application_contribution::WorthQueryInstalledOutputReadinessRoutes,
        next_output_producer_attempt: AtomicU64,
        mutation_partitions: mutation_partition::ApplicationMutationPartitions,
        output_demands: super::super::super::application_output_demand::WorthQueryOutputDemandRegistry,
        recovered_outputs: super::super::super::application_output_demand::WorthQueryRecoveredOutputs,
        workflow_coverage: super::super::super::workflow::adoption::WorkflowVocabularyCoverageRegistry,
        installed_operations: Arc<worth_query_installation::facade::InstalledOperationCatalog>,
    ) -> Self;

    fn execution_option_mut(&mut self) -> &mut Option<WorthQueryExecutionRuntime>;
    fn execution_ref_and_bridge_mut(
        &mut self,
    ) -> (
        Option<&WorthQueryExecutionRuntime>,
        &mut super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
    );
    fn application_readiness_schema_token_ref(&self) -> &str;
    fn authorization_ref(&self) -> &Arc<WorthQueryInstalledAuthorizationRegistry>;
    fn authorization_clock_ref(&self) -> &Arc<WorthQueryRuntimeClock>;
    fn relational_branch_identity_ref(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchIdentity;
    fn bridge_ref(
        &self,
    ) -> &super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge;
    fn bridge_mut(
        &mut self,
    ) -> &mut super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge;
    fn product_runtime_ref(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;
    fn product_runtime_mut(
        &mut self,
    ) -> &mut crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;
    fn recovered_program_source_delivery_ref(
        &self,
    ) -> &super::super::super::provider::recovered_program_source_delivery::RecoveredProgramSourceDeliveryRegistration;
    fn basis_leases_ref(
        &self,
    ) -> &super::super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistry;
    fn granular_invalidation_ref(
        &self,
    ) -> &super::super::super::WorthQueryGranularInvalidationInstallation;
    fn granular_invalidation_mut(
        &mut self,
    ) -> &mut super::super::super::WorthQueryGranularInvalidationInstallation;
    fn primary_provider_ref(&self) -> &super::super::super::provider::PrimaryProviderCustody;
    fn primary_provider_mut(
        &mut self,
    ) -> &mut super::super::super::provider::PrimaryProviderCustody;
    fn primary_graph_authority_option_mut(
        &mut self,
    ) -> &mut Option<home_retirement::PrimaryGraphParticipationCustody>;
    fn primary_graph_authority_option_mut_and_primary_provider_ref(
        &mut self,
    ) -> (
        &mut Option<home_retirement::PrimaryGraphParticipationCustody>,
        &super::super::super::provider::PrimaryProviderCustody,
    );
    fn source_meanings_option_mut(
        &mut self,
    ) -> &mut Option<super::super::super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry>;
    fn next_external_dispatch_attempt_ref(&self) -> &AtomicU64;
    fn external_effect_transport_ref(
        &self,
    ) -> &std::sync::OnceLock<Arc<dyn WorthQueryExternalEffectTransport>>;
    fn inbound_verifiers_ref(
        &self,
    ) -> &std::sync::Mutex<
        std::collections::BTreeMap<
            String,
            Arc<inbound_occurrence::WorthQueryInstalledInboundVerifier>,
        >,
    >;
    fn inbound_verifiers_mut(
        &mut self,
    ) -> &mut std::sync::Mutex<
        std::collections::BTreeMap<
            String,
            Arc<inbound_occurrence::WorthQueryInstalledInboundVerifier>,
        >,
    >;
    fn inbound_custody_ref(
        &self,
    ) -> &Arc<
        std::sync::Mutex<
            crate::domain_computation::application_aftermath::WorthQueryInboundCustody,
        >,
    >;
    fn transport_completion_custody_ref(
        &self,
    ) -> &std::sync::Mutex<inbound_publication::InstalledTransportCompletionCustody>;
    fn inbound_maintenance_turn_ref(&self) -> &AtomicU64;
    fn dispatch_journal_ref(
        &self,
    ) -> &Arc<
        crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal,
    >;
    fn recovery_handles_ref(&self) -> &Arc<WorthQueryRecoveryHandleRegistry>;
    fn output_producer_routes_ref(
        &self,
    ) -> &super::super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes;
    fn output_readiness_routes_ref(
        &self,
    ) -> &super::super::super::application_contribution::WorthQueryInstalledOutputReadinessRoutes;
    fn next_output_producer_attempt_ref(&self) -> &AtomicU64;
    fn mutation_partitions_ref(&self) -> &mutation_partition::ApplicationMutationPartitions;
    fn output_demands_ref(
        &self,
    ) -> &super::super::super::application_output_demand::WorthQueryOutputDemandRegistry;
    fn output_demands_mut(
        &mut self,
    ) -> &mut super::super::super::application_output_demand::WorthQueryOutputDemandRegistry;
    fn recovered_outputs_ref(
        &self,
    ) -> &super::super::super::application_output_demand::WorthQueryRecoveredOutputs;
    fn recovered_outputs_mut(
        &mut self,
    ) -> &mut super::super::super::application_output_demand::WorthQueryRecoveredOutputs;
    fn workflow_coverage_ref(
        &self,
    ) -> &super::super::super::workflow::adoption::WorkflowVocabularyCoverageRegistry;
    fn workflow_coverage_mut(
        &mut self,
    ) -> &mut super::super::super::workflow::adoption::WorkflowVocabularyCoverageRegistry;
    fn installed_operations_ref(
        &self,
    ) -> &Arc<worth_query_installation::facade::InstalledOperationCatalog>;
}
