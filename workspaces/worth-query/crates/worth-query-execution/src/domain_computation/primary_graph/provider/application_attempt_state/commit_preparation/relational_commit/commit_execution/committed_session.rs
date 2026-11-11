//! Access to the actual performed application session.
use super::*;
impl WorthQueryCommittedApplicationSession {
    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn retain_recovered_producer_facts(
        &self,
        evidence: &super::super::WorthQueryPrimaryGraphCommitEvidence,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::RecoveredProducerOutputFacts>{
        use crate::domain_computation::primary_graph::application_output_demand::RecoveredProducerOutputFacts;
        std::sync::Arc::new(RecoveredProducerOutputFacts {
            publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication::from_receipt(self.product_publication.clone()),
            changes: evidence.committed_changes().clone(),
            outcome: self.attempt.outcome_identity(),
            correspondence: evidence.retain_output_correspondence(),
            authority: evidence.retain_original_authority(),
            inputs: evidence.performed_input_facts().cloned(),
            fields: evidence.performed_output_fields().cloned(),
            unrebased_own_effect: evidence.rebased_source_facts().err().map(|failure| failure.own_effect),
            settlement: self.prepared_lineage_slot.as_ref().map(|slot| slot.identity().clone()),
            _memory: memory,
        })
    }
    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn take_predicate_preparation(&mut self) -> Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>{
        self.predicate.take()
    }
    pub(in crate::domain_computation::primary_graph::provider) fn take_recovered_output_source(
        &self,
    ) -> crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource{
        crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource::take_original(&self.product_publication)
    }
    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn take_prepared_touched_records(
        &mut self,
    ) -> PreparedTouchedRecords {
        self.prepared_touched_records
            .take()
            .expect("pre-effect native touched-record receipt storage follows publication")
    }
    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn take_source_fact_rebase(
        &mut self,
    ) -> super::super::PreparedSourceFactRebase {
        self.source_fact_rebase
            .take()
            .expect("pre-effect source-fact rebase capacity follows the committed candidate")
    }
    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) const fn attempt(
        &self,
    ) -> &WorthQueryPrimaryGraphApplicationAttempt {
        &self.attempt
    }

    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) const fn work(
        &self,
    ) -> WorthQueryPrimaryMutationWorkCounters {
        self.work
    }

    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) const fn index_maintenance_work(
        &self,
    ) -> worth_relational::facade::indexes::DerivedIndexMaintenanceWork {
        self.index_maintenance_work
    }

    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) const fn retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }

    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) const fn preimage_retention_work(
        &self,
    ) -> WorthQueryPreImageRetentionWork {
        self.preimage_retention_work
    }

    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn changed_records(
        &self,
    ) -> &[worth_relational::facade::transactions::RecordRef] {
        self.own_records
            .as_ref()
            .map(|records| records.elements())
            .unwrap_or(&self.committed.changed_records)
    }
    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn coupled_commit(
        &self,
    ) -> Option<&worth_relational::facade::transactions::CommitResult> {
        (!self.members.is_empty() || self.own_records.is_some()).then_some(self.committed.as_ref())
    }
    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn output_correspondence(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        match &self.own_publication {
            Some(publication) => self
                .attempt
                .seal_output_correspondence_from_publication(publication),
            None => self.attempt.seal_output_correspondence(&self.committed),
        }
    }
    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn committed(
        &self,
    ) -> &worth_relational::facade::transactions::CommitResult {
        self.committed.as_ref()
    }

    pub(in crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit) fn publish_and_encode(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &WorthQueryPrimaryGraphProvider,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        evidence: super::super::WorthQueryPrimaryGraphCommitEvidence,
        predicate: &mut Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>,
        retired: &mut crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationRetirement,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        let published = publication::publish(
            execution, provider, runtime, self, evidence, predicate, retired,
        )?;
        publication::encode(provider, published)
    }
}
