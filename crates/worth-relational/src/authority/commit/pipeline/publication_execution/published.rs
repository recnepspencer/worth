//! Decompose the performed publication without losing original writer evidence.
use super::PublishedCommitExecution;
impl PublishedCommitExecution {
    pub(crate) fn snapshot_id(&self) -> crate::snapshots::data::SnapshotId {
        self.publication_snapshot.snapshot_id()
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn into_result_operands(self) -> (
        crate::transactions::data::primitives::TransactionId,
        crate::transactions::data::outcomes::commit_result::CommitPhaseTiming,
        crate::transactions::data::commit_log::CommitLog,
        Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
        crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
        crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
        crate::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
        Vec<crate::validation::engine::InvariantExecutionResult>,
        crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        crate::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
        std::sync::Arc<crate::history::data::CanonicalCommitEnvelope>,
        crate::publication::patch::data::PatchStreamPosition,
        crate::snapshots::data::SnapshotHandle,
        Option<crate::mvcc::publication::application_publication::RelationalCoupledAuthoredSourceEffects>,
        Vec<crate::transactions::data::aspect_traces::AspectEvaluationTrace>,
        Vec<crate::transactions::data::aspect_traces::AspectEmissionTrace>,
    ){
        let (
            transaction_id,
            phase_timing,
            commit_log,
            strategy_artifacts,
            diagnostics,
            complexity_before,
            prior_complexity_delta,
        ) = self.admitted.into_inner().into_result_parts();
        (
            transaction_id,
            phase_timing,
            commit_log,
            strategy_artifacts,
            diagnostics,
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
