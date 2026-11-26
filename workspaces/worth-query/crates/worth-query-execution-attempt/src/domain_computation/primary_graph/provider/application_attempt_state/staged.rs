use super::{WorthQueryProposedFact, WorthQueryStagedApplicationAttempt};

impl WorthQueryStagedApplicationAttempt<'_> {
    pub fn appendix_attempt(&self) -> &super::WorthQueryPrimaryGraphApplicationAttempt {
        self.attempt
    }

    pub fn content_attachments(
        &self,
    ) -> &worth_relational::facade::content::RelationalContentAttachments {
        self.attempt.content_attachments()
    }

    pub fn authored_restoration_anchor(
        &self,
    ) -> Result<
        Option<(
            worth_relational::facade::history::CommitId,
            worth_relational::facade::identity::EntityId,
            worth_relational::facade::transactions::CreatedEntityRef,
            Vec<(
                worth_relational::facade::identity::EntityId,
                worth_relational::facade::transactions::CreatedEntityRef,
            )>,
        )>,
        crate::domain_computation::WorthQueryInvariantExecutionFailure,
    > {
        let Some(pending) = self.attempt.aftermath_causality() else {
            return Ok(None);
        };
        let prepared = pending.prepared_correction();
        let selected = prepared
            .authored_restoration_anchor(
                &self.attempt.native_publication_roles().collect::<Vec<_>>(),
                self.batch(),
                self.validation_request(),
            )
            .map_err(|_| super::super::invariant_execution::touch_failure())?;
        Ok(selected.map(|(old, new)| {
            (
                prepared.source_commit().commit_id,
                old,
                new,
                prepared.authored_restoration_co_anchors(
                    &self.attempt.native_publication_roles().collect::<Vec<_>>(),
                ),
            )
        }))
    }

    pub fn validation_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.attempt.affinity().publication_request()
    }

    /// The operation's single trusted admission instant, handed to Relational
    /// so every installed invariant observes the decision's own reading.
    pub const fn admitted_at(
        &self,
    ) -> Result<
        worth_foundational::facade::CanonicalTimestamp,
        crate::domain_computation::runtime_time::WorthQueryRuntimeTimeDenial,
    > {
        self.attempt.affinity().admitted_at()
    }

    pub fn overlay_identity(&self) -> &str {
        self.overlay.identity()
    }

    pub fn overlay_facts(&self) -> &[WorthQueryProposedFact] {
        self.overlay.facts()
    }

    pub fn expected_step_count(&self) -> usize {
        self.attempt.expected_steps().len()
    }

    pub const fn expected_step_preparation_work(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryExpectedEffectStepPreparationWork{
        self.attempt.expected_step_preparation_work()
    }

    pub fn batch(&self) -> &worth_relational::facade::transactions::WorkerIntentBatch {
        self.attempt.batch()
    }

    /// The naming plan the candidate transaction attaches; `Err` when the
    /// attempt's declared output names were never lowered.
    pub fn naming_plan(
        &self,
    ) -> Result<Option<&worth_relational::facade::lineage::CommitNamingPlan>, ()> {
        self.attempt.naming_plan()
    }

    pub fn branch(&self) -> &worth_relational::facade::history::BranchId {
        self.attempt.affinity().branch()
    }

    pub fn product_publication(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding
{
        self.attempt.affinity().product_publication()
    }

    pub fn decision_fact_count(&self) -> usize {
        self.attempt.decision_fact_count()
    }

    pub fn invariant_requirements(
        &self,
    ) -> &[worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement] {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .invariant_requirements()
    }

    pub fn producer_required_invariants(
        &self,
    ) -> &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement]
    {
        self.attempt.producer_required_invariants()
    }

    pub fn aftermath_causality(
        &self,
    ) -> Option<
        &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    > {
        self.attempt.aftermath_causality()
    }

    pub fn application_graph_reads(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationGraphReadContract> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_graph_reads()
    }

    pub fn application_touches(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationTouchContract> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_touches()
    }

    pub fn application_read_touch_overlap(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_read_touch_overlap()
    }
}
