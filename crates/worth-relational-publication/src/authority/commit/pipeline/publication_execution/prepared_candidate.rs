//! Read and consume the same funded, prepared publication candidate.
use super::{PreparedCommitPublicationCompletion, PreparedCommitPublicationExecution};
use worth_relational_contracts::internal_port as contracts;
impl PreparedCommitPublicationExecution {
    pub(crate) fn application_unchanged(&self) -> bool {
        self.publication.value().application_unchanged
    }
    pub(crate) fn appendix_adoption(
        &self,
    ) -> Option<&worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>{
        self.admitted.value().validation_input().appendix_adoption()
    }

    pub(crate) fn canonical_envelope(
        &self,
    ) -> &std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>{
        self.publication.value().canonical_envelope()
    }
    pub(crate) fn caller_association(
        &self,
    ) -> Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>{
        self.publication.value().caller_association()
    }
    pub(crate) fn descriptive_touches(
        &self,
    ) -> &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouchGraph{
        self.publication.value().descriptive_touches()
    }
    pub(crate) fn changed_record_count(&self) -> usize {
        self.publication.value().changed_record_count()
    }
    pub(crate) fn changed_records(
        &self,
    ) -> &[contracts::transactions::data::primitives::RecordRef] {
        &self.publication.value().changed_records
    }
    pub(crate) fn attach_prepared_indexes(
        &mut self,
        prepared: worth_relational_indexes::internal_port::indexes::authority::maintenance::candidate_publication::PreparedCandidateIndexPublication,
    ) -> Result<(), contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind> {
        if self.prepared_indexes.is_some() {
            return Err(
                contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind::CandidateIndexesAlreadyPrepared,
            );
        }
        if !prepared.matches_root(self.prepared_root()) {
            return Err(contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind::CommitMismatch);
        }
        self.prepared_indexes = Some(prepared);
        Ok(())
    }
    pub(crate) fn recovered_codec_publication_custody(
        &self,
    ) -> Option<&std::sync::Arc<worth_relational_durability::internal_port::durability::authority::recovery::codec_transition::publication_custody::RecoveredCodecPublicationCustody>>
{
        self.admitted
            .value()
            .validation_input()
            .recovered_codec_publication_custody()
    }

    pub(crate) fn home_step(
        &self,
    ) -> Option<&std::sync::Arc<worth_relational_branch::internal_port::branch::home_program_step::AdmittedRelationalHomeProgramStep>>
{
        self.admitted.value().validation_input().home_step()
    }

    pub(crate) fn home_index_definitions(
        &self,
    ) -> Option<
        std::sync::Arc<
            worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::definition_preparation::PreparedRelationalIndexDefinitions,
        >,
    >{
        self.admitted
            .value()
            .validation_input()
            .prepared_home_indexes
            .clone()
    }

    pub(crate) fn covers_home_definitions(
        &self,
        definitions: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::definition_preparation::PreparedRelationalIndexDefinitions,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, worth_execution::WorkCeilingDenial> {
        match &self.prepared_indexes {
            Some(prepared) => prepared.covers_home_definitions(definitions, execution),
            None => Ok(false),
        }
    }

    pub(crate) fn has_prepared_indexes(&self) -> bool {
        self.prepared_indexes.is_some()
    }

    pub(crate) fn reservation_count(&self) -> usize {
        self.publication.value().reservation_count()
    }

    pub(crate) fn prepared_root(
        &self,
    ) -> &std::sync::Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>
    {
        self.publication.value().prepared_root()
    }

    pub(crate) fn take_live_publication_inputs(
        &mut self,
    ) -> crate::publication::live_publication_inputs::BoundLivePublicationInputs {
        self.publication
            .value_mut()
            .live_inputs
            .take()
            .expect("registration consumes the original prepared route inputs")
    }

    /// The parents its commit receipt names, resolved before publication.
    pub(crate) fn successor_parent_count(&self) -> usize {
        self.history.commit_reference.parents.len()
    }

    pub(crate) fn is_retained_entity_selection(&self) -> bool {
        self.admitted.value().semantic_plan().selection().is_some()
    }

    pub(crate) fn release_transaction_retention(&mut self) {
        self.admitted.value_mut().release_transaction_retention();
    }

    pub(crate) fn append_diagnostics(
        &mut self,
        diagnostics: Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
    ) {
        self.admitted.value_mut().append_diagnostics(diagnostics);
    }

    pub(crate) fn split(
        self,
        published_snapshot_slot: worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
    ) -> (
        crate::mvcc::publication::PreparedCanonicalBranchMovement,
        PreparedCommitPublicationCompletion,
    ) {
        let (movement, publication) = self.publication.into_inner().split();
        let completion = PreparedCommitPublicationCompletion {
            source_interpretation: self.source_interpretation,
            admitted: self.admitted,
            public_structural_summary: self.public_structural_summary,
            invariant_executions: self.invariant_executions,
            created_entities: self.created_entities,
            created_relations: self.created_relations,
            history: self.history,
            publication,
            publication_snapshot: self.publication_snapshot,
            source_effects: self.source_effects,
            aspect_evaluation_traces: self.aspect_evaluation_traces,
            aspect_emission_traces: self.aspect_emission_traces,
            published_snapshot_slot,
            prepared_indexes: self.prepared_indexes,
        };
        (movement, completion)
    }

    pub(crate) fn materialization_counts(&self) -> (u64, u64) {
        self.publication.value().materialization_counts()
    }
}
