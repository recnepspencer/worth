//! Read and consume the same funded, prepared publication candidate.
use super::{PreparedCommitPublicationCompletion, PreparedCommitPublicationExecution};
impl PreparedCommitPublicationExecution {
    pub(crate) fn canonical_envelope(
        &self,
    ) -> &std::sync::Arc<crate::history::data::CanonicalCommitEnvelope> {
        self.publication.value().canonical_envelope()
    }
    pub(crate) fn owned_graph_selection_model(&self) -> Option<crate::identity::data::EntityId> {
        self.admitted
            .value()
            .semantic_plan()
            .graph_selection()
            .map(|graph| graph.model())
    }

    pub(crate) fn caller_association(&self) -> Option<crate::mvcc::RelationalCallerAssociation> {
        self.publication.value().caller_association()
    }
    pub(crate) fn descriptive_touches(
        &self,
    ) -> &crate::history::data::RelationalDescriptiveTouchGraph {
        self.publication.value().descriptive_touches()
    }
    pub(crate) fn changed_record_count(&self) -> usize {
        self.publication.value().changed_record_count()
    }
    pub(crate) fn changed_records(&self) -> &[crate::transactions::data::RecordRef] {
        &self.publication.value().changed_records
    }
    pub(crate) fn attach_prepared_indexes(
        &mut self,
        prepared: crate::indexes::PreparedCandidateIndexPublication,
    ) -> Result<(), crate::indexes::data::DerivedIndexMaintenanceDenialKind> {
        if self.prepared_indexes.is_some() {
            return Err(
                crate::indexes::data::DerivedIndexMaintenanceDenialKind::CandidateIndexesAlreadyPrepared,
            );
        }
        if !prepared.matches_root(self.prepared_root()) {
            return Err(crate::indexes::data::DerivedIndexMaintenanceDenialKind::CommitMismatch);
        }
        self.prepared_indexes = Some(prepared);
        Ok(())
    }
    pub(crate) fn recovered_codec_publication_custody(
        &self,
    ) -> Option<&std::sync::Arc<crate::durability::RecoveredCodecPublicationCustody>> {
        self.admitted
            .value()
            .validation_input()
            .recovered_codec_publication_custody()
    }

    pub(crate) fn home_step(
        &self,
    ) -> Option<&std::sync::Arc<crate::runtime::AdmittedRelationalHomeProgramStep>> {
        self.admitted.value().validation_input().home_step()
    }

    pub(crate) fn home_index_definitions(
        &self,
    ) -> Option<std::sync::Arc<crate::runtime::PreparedRelationalIndexDefinitions>> {
        self.admitted
            .value()
            .validation_input()
            .home_target()
            .and_then(|target| target.index_definitions.clone())
    }

    pub(crate) fn covers_home_definitions(
        &self,
        definitions: &crate::runtime::PreparedRelationalIndexDefinitions,
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

    pub(crate) fn prepared_root(&self) -> &std::sync::Arc<crate::branch::RelationalBranchRoot> {
        self.publication.value().prepared_root()
    }

    pub(crate) fn is_retained_entity_selection(&self) -> bool {
        self.admitted.value().semantic_plan().selection().is_some()
    }

    pub(crate) fn release_transaction_retention(&mut self) {
        self.admitted.value_mut().release_transaction_retention();
    }

    pub(crate) fn append_diagnostics(
        &mut self,
        diagnostics: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    ) {
        self.admitted.value_mut().append_diagnostics(diagnostics);
    }

    pub(crate) fn split(
        self,
        published_snapshot_slot: crate::runtime::PublishedSnapshotSlotReservation,
    ) -> (
        crate::mvcc::PreparedCanonicalBranchMovement,
        PreparedCommitPublicationCompletion,
    ) {
        let (movement, publication) = self.publication.into_inner().split();
        let completion = PreparedCommitPublicationCompletion {
            source_interpretation: self.source_interpretation,
            admitted: self.admitted,
            public_structural_summary: self.public_structural_summary,
            invariant_executions: self.invariant_executions,
            version_id: self.version_id,
            created_entities: self.created_entities,
            created_relations: self.created_relations,
            history: self.history,
            publication,
            publication_snapshot: self.publication_snapshot,
            aspect_evaluation_traces: self.aspect_evaluation_traces,
            aspect_emission_traces: self.aspect_emission_traces,
            published_snapshot_slot,
            prepared_indexes: self.prepared_indexes,
        };
        (movement, completion)
    }

    #[cfg(test)]
    pub(crate) fn materialization_counts(&self) -> (u64, u64) {
        self.publication.value().materialization_counts()
    }
}
