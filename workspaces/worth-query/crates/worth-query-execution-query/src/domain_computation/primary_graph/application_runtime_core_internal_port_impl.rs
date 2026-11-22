//! Implementation of `ApplicationRuntimeCoreInternalPort` for
//! `WorthQueryApplicationRuntimeCore`.

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use super::super::super::*;
use super::super::WorthQueryApplicationRuntimeCore;
use super::ApplicationRuntimeCoreInternalPort;

impl ApplicationRuntimeCoreInternalPort for WorthQueryApplicationRuntimeCore {
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
        bridge: super::super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
        product_runtime: crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
        recovered_program_source_delivery: super::super::super::super::provider::recovered_program_source_delivery::RecoveredProgramSourceDeliveryRegistration,
        basis_leases: super::super::super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistry,
        granular_invalidation: super::super::super::super::WorthQueryGranularInvalidationInstallation,
        primary_provider: super::super::super::super::provider::PrimaryProviderCustody,
        primary_graph_authority: Option<home_retirement::PrimaryGraphParticipationCustody>,
        source_meanings: Option<super::super::super::super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry>,
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
        output_producer_routes: super::super::super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes,
        output_readiness_routes: super::super::super::super::application_contribution::WorthQueryInstalledOutputReadinessRoutes,
        next_output_producer_attempt: AtomicU64,
        mutation_partitions: mutation_partition::ApplicationMutationPartitions,
        output_demands: super::super::super::super::application_output_demand::WorthQueryOutputDemandRegistry,
        recovered_outputs: super::super::super::super::application_output_demand::WorthQueryRecoveredOutputs,
        workflow_coverage: super::super::super::super::workflow::adoption::WorkflowVocabularyCoverageRegistry,
        installed_operations: Arc<worth_query_installation::facade::InstalledOperationCatalog>,
    ) -> Self {
        Self {
            execution,
            application_readiness_schema_token,
            publication,
            checkpoint_restore_work,
            authorization,
            authorization_clock,
            relational_branch_identity,
            bridge,
            product_runtime,
            recovered_program_source_delivery,
            basis_leases,
            granular_invalidation,
            primary_provider,
            primary_graph_authority,
            source_meanings,
            next_external_dispatch_attempt,
            external_effect_transport,
            inbound_verifiers,
            inbound_custody,
            transport_completion_custody,
            inbound_maintenance_turn,
            dispatch_journal,
            recovery_handles,
            output_producer_routes,
            output_readiness_routes,
            next_output_producer_attempt,
            mutation_partitions,
            output_demands,
            recovered_outputs,
            workflow_coverage,
            installed_operations,
        }
    }

    fn execution_option_mut(&mut self) -> &mut Option<WorthQueryExecutionRuntime> {
        &mut self.execution
    }

    fn execution_ref_and_bridge_mut(
        &mut self,
    ) -> (
        Option<&WorthQueryExecutionRuntime>,
        &mut super::super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
    ) {
        (self.execution.as_ref(), &mut self.bridge)
    }

    fn application_readiness_schema_token_ref(&self) -> &str {
        &self.application_readiness_schema_token
    }

    fn authorization_ref(&self) -> &Arc<WorthQueryInstalledAuthorizationRegistry> {
        &self.authorization
    }

    fn authorization_clock_ref(&self) -> &Arc<WorthQueryRuntimeClock> {
        &self.authorization_clock
    }

    fn relational_branch_identity_ref(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchIdentity {
        &self.relational_branch_identity
    }

    fn bridge_ref(
        &self,
    ) -> &super::super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge {
        &self.bridge
    }

    fn bridge_mut(
        &mut self,
    ) -> &mut super::super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge {
        &mut self.bridge
    }

    fn product_runtime_ref(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        &self.product_runtime
    }

    fn product_runtime_mut(
        &mut self,
    ) -> &mut crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        &mut self.product_runtime
    }

    fn recovered_program_source_delivery_ref(
        &self,
    ) -> &super::super::super::super::provider::recovered_program_source_delivery::RecoveredProgramSourceDeliveryRegistration{
        &self.recovered_program_source_delivery
    }

    fn basis_leases_ref(
        &self,
    ) -> &super::super::super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistry{
        &self.basis_leases
    }

    fn granular_invalidation_ref(
        &self,
    ) -> &super::super::super::super::WorthQueryGranularInvalidationInstallation {
        &self.granular_invalidation
    }

    fn granular_invalidation_mut(
        &mut self,
    ) -> &mut super::super::super::super::WorthQueryGranularInvalidationInstallation {
        &mut self.granular_invalidation
    }

    fn primary_provider_ref(
        &self,
    ) -> &super::super::super::super::provider::PrimaryProviderCustody {
        &self.primary_provider
    }

    fn primary_provider_mut(
        &mut self,
    ) -> &mut super::super::super::super::provider::PrimaryProviderCustody {
        &mut self.primary_provider
    }

    fn primary_graph_authority_option_mut(
        &mut self,
    ) -> &mut Option<home_retirement::PrimaryGraphParticipationCustody> {
        &mut self.primary_graph_authority
    }

    fn primary_graph_authority_option_mut_and_primary_provider_ref(
        &mut self,
    ) -> (
        &mut Option<home_retirement::PrimaryGraphParticipationCustody>,
        &super::super::super::super::provider::PrimaryProviderCustody,
    ) {
        (&mut self.primary_graph_authority, &self.primary_provider)
    }

    fn source_meanings_option_mut(
        &mut self,
    ) -> &mut Option<super::super::super::super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry>{
        &mut self.source_meanings
    }

    fn next_external_dispatch_attempt_ref(&self) -> &AtomicU64 {
        &self.next_external_dispatch_attempt
    }

    fn external_effect_transport_ref(
        &self,
    ) -> &std::sync::OnceLock<Arc<dyn WorthQueryExternalEffectTransport>> {
        &self.external_effect_transport
    }

    fn inbound_verifiers_ref(
        &self,
    ) -> &std::sync::Mutex<
        std::collections::BTreeMap<
            String,
            Arc<inbound_occurrence::WorthQueryInstalledInboundVerifier>,
        >,
    > {
        &self.inbound_verifiers
    }

    fn inbound_verifiers_mut(
        &mut self,
    ) -> &mut std::sync::Mutex<
        std::collections::BTreeMap<
            String,
            Arc<inbound_occurrence::WorthQueryInstalledInboundVerifier>,
        >,
    > {
        &mut self.inbound_verifiers
    }

    fn inbound_custody_ref(
        &self,
    ) -> &Arc<
        std::sync::Mutex<
            crate::domain_computation::application_aftermath::WorthQueryInboundCustody,
        >,
    > {
        &self.inbound_custody
    }

    fn transport_completion_custody_ref(
        &self,
    ) -> &std::sync::Mutex<inbound_publication::InstalledTransportCompletionCustody> {
        &self.transport_completion_custody
    }

    fn inbound_maintenance_turn_ref(&self) -> &AtomicU64 {
        &self.inbound_maintenance_turn
    }

    fn dispatch_journal_ref(
        &self,
    ) -> &Arc<
        crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal,
    >{
        &self.dispatch_journal
    }

    fn recovery_handles_ref(&self) -> &Arc<WorthQueryRecoveryHandleRegistry> {
        &self.recovery_handles
    }

    fn output_producer_routes_ref(
        &self,
    ) -> &super::super::super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes{
        &self.output_producer_routes
    }

    fn output_readiness_routes_ref(
        &self,
    ) -> &super::super::super::super::application_contribution::WorthQueryInstalledOutputReadinessRoutes{
        &self.output_readiness_routes
    }

    fn next_output_producer_attempt_ref(&self) -> &AtomicU64 {
        &self.next_output_producer_attempt
    }

    fn mutation_partitions_ref(&self) -> &mutation_partition::ApplicationMutationPartitions {
        &self.mutation_partitions
    }

    fn output_demands_ref(
        &self,
    ) -> &super::super::super::super::application_output_demand::WorthQueryOutputDemandRegistry
    {
        &self.output_demands
    }

    fn output_demands_mut(
        &mut self,
    ) -> &mut super::super::super::super::application_output_demand::WorthQueryOutputDemandRegistry
    {
        &mut self.output_demands
    }

    fn recovered_outputs_ref(
        &self,
    ) -> &super::super::super::super::application_output_demand::WorthQueryRecoveredOutputs {
        &self.recovered_outputs
    }

    fn recovered_outputs_mut(
        &mut self,
    ) -> &mut super::super::super::super::application_output_demand::WorthQueryRecoveredOutputs
    {
        &mut self.recovered_outputs
    }

    fn workflow_coverage_ref(
        &self,
    ) -> &super::super::super::super::workflow::adoption::WorkflowVocabularyCoverageRegistry {
        &self.workflow_coverage
    }

    fn workflow_coverage_mut(
        &mut self,
    ) -> &mut super::super::super::super::workflow::adoption::WorkflowVocabularyCoverageRegistry
    {
        &mut self.workflow_coverage
    }

    fn installed_operations_ref(
        &self,
    ) -> &Arc<worth_query_installation::facade::InstalledOperationCatalog> {
        &self.installed_operations
    }
}
