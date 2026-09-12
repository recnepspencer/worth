use super::{WorthQueryProposedFact, WorthQueryStagedApplicationAttempt};

impl WorthQueryStagedApplicationAttempt<'_> {
    pub(in crate::domain_computation::primary_graph::provider) fn overlay_identity(&self) -> &str {
        self.overlay.identity()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn overlay_facts(
        &self,
    ) -> &[WorthQueryProposedFact] {
        self.overlay.facts()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn expected_step_count(
        &self,
    ) -> usize {
        self.attempt.expected_steps().len()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn batch(
        &self,
    ) -> &worth_relational::facade::transactions::WorkerIntentBatch {
        self.attempt.batch()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn branch(
        &self,
    ) -> &worth_relational::facade::history::BranchId {
        self.attempt.affinity().branch()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn product_publication(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding
    {
        self.attempt.affinity().product_publication()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn decision_fact_count(
        &self,
    ) -> usize {
        self.attempt.decision_fact_count()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn invariant_requirements(
        &self,
    ) -> &[worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement] {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .invariant_requirements()
    }

    pub(in crate::domain_computation::primary_graph::provider) const fn validator_work_admission(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryCandidateValidatorWorkAdmission{
        self.attempt.validator_work_admission()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn aftermath_causality(
        &self,
    ) -> Option<
        &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    > {
        self.attempt.aftermath_causality()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn application_graph_reads(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationGraphReadContract> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_graph_reads()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn application_touches(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationTouchContract> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_touches()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn application_read_touch_overlap(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_read_touch_overlap()
    }
}
