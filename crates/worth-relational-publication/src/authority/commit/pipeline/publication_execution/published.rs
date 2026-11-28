//! Decompose the performed publication without losing original writer evidence.
use super::PublishedCommitExecution;
use worth_relational_contracts::internal_port as contracts;
impl PublishedCommitExecution {
    pub(crate) fn snapshot_id(&self) -> contracts::snapshots::data::coordinates::SnapshotId {
        self.publication_snapshot.snapshot_id()
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn into_result_operands(self) -> (
        contracts::transactions::data::primitives::TransactionId,
        crate::transactions::data::outcomes::commit_result::CommitPhaseTiming,
        contracts::transactions::data::commit_log::CommitLog,
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
        contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
        Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>,
        contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
        std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>,
        contracts::publication::patch::data::patch_position::PatchStreamPosition,
        worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
        Option<crate::mvcc::publication::application_publication::RelationalCoupledAuthoredSourceEffects>,
        Vec<contracts::transactions::data::aspect_traces::AspectEvaluationTrace>,
        Vec<contracts::transactions::data::aspect_traces::AspectEmissionTrace>,
    ){
        let (
            transaction_id,
            phase_timing,
            commit_log,
            complexity_before,
            prior_complexity_delta,
        ) = self.admitted.into_inner().into_result_parts();
        (
            transaction_id,
            phase_timing,
            commit_log,
            complexity_before,
            prior_complexity_delta,
            self.public_structural_summary,
            self.invariant_executions,
            self.created_entities,
            self.created_relations,
            self.canonical_commit_envelope,
            self.patch_position,
            self.publication_snapshot,
            self.source_effects,
            self.aspect_evaluation_traces,
            self.aspect_emission_traces,
        )
    }
}
