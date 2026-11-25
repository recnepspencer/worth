//! Sealed test doors over this module's private members.
use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn installed_producers_for_test(
        &self,
    ) -> &super::super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<
        Schema,
    > {
        &self.inner.installed_producers
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn installed_schema_for_test(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        &self.inner.installed_schema
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn program_support_for_test(
        &self,
    ) -> &Option<super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        &self.inner.program_support
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn program_support_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>
    {
        &mut self.inner.program_support
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn installed_program_support_for_test(
        &self,
    ) -> Option<&super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        Self::installed_program_support(self)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn mutation_projection_for_test(
        &self,
    ) -> &super::super::WorthQueryApplicationInvariantProjectionAuthority<Schema> {
        &self.inner.mutation_projection
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn program_required_operations_for_test(&self) -> &std::collections::HashSet<TypeId> {
        &self.inner.program_required_operations
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    #[doc(hidden)]
    pub fn graph_work_resource_support_ref_for_test(
    &self) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot{
        Self::graph_work_resource_support_ref(self)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn runtime_for_test(&self) -> &WorthQueryApplicationRuntimeCore {
        &self.inner.runtime
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn mutation_handlers_mut_for_test(
        &mut self,
    ) -> &mut super::super::handler::InstalledMutationHandlerRegistry<Schema> {
        &mut self.inner.mutation_handlers
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn runtime_mut_for_test(&mut self) -> &mut WorthQueryApplicationRuntimeCore {
        &mut self.inner.runtime
    }

    #[doc(hidden)]
    pub fn observe_next_correction_pin_transfer_for_test(
        &self,
        observer: Box<dyn FnOnce() + Send>,
    ) {
        self.inner
            .observe_next_correction_pin_transfer_for_test(observer);
    }

    #[doc(hidden)]
    pub fn pause_branch_adoption_stage_for_test(
        &self,
        stage: worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::WorthQueryBranchAdoptionStageForTest,
    ) -> worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::WorthQueryBranchAdoptionStagePauseForTest{
        self.inner.pause_branch_adoption_stage_for_test(stage)
    }

    #[doc(hidden)]
    pub fn observe_branch_adoption_preparations_for_test(
        &self,
    ) -> worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::WorthQueryBranchAdoptionPreparationObserverForTest{
        self.inner.observe_branch_adoption_preparations_for_test()
    }

    #[doc(hidden)]
    pub fn cold_adoption_readmission_summary_for_test(
        &self,
    ) -> super::super::query_obligations::history::QueryColdAdoptionReadmissionSummary {
        self.inner.cold_adoption_readmission_summary_for_test()
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn lose_next_commit_answer_for_test(&self) {
        self.inner.lose_next_commit_answer_for_test();
    }

    #[doc(hidden)]
    pub fn refuse_next_conditional_decision_publication_for_test(&self) {
        self.conditional_cleanup
            .routes
            .refuse_next_decision_publication
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub(in crate::domain_computation::primary_graph) fn take_refuse_next_decision_publication(
        &self,
    ) -> bool {
        self.conditional_cleanup
            .routes
            .refuse_next_decision_publication
            .swap(false, std::sync::atomic::Ordering::AcqRel)
    }

    #[doc(hidden)]
    pub fn pause_after_commit_before_answer_for_test(
        &self,
    ) -> worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::WorthQueryCommitAnswerPauseForTest{
        self.inner.pause_after_commit_before_answer_for_test()
    }
}

mod basis_and_workflow_cache;
mod commit_and_collection_pauses;
mod commit_validation_and_external_settlement;
mod coupled_producer_observations;
mod inbound_cleanup_and_transport;
mod output_demand_and_source_selection;
mod ownership_locality;
mod ownership_probes;
mod ownership_sources_and_cycles;
mod product_branch_and_workflow_basis;
mod query_and_disclosure;
