//! Decompose the performed publication without losing original writer evidence.
use super::PublishedCommitExecution;
impl PublishedCommitExecution {
    #[allow(clippy::type_complexity)]
    pub(in crate::authority::commit::pipeline) fn into_parts(
        self,
    ) -> (
        super::super::phase_backing::CommitPhaseBacking<
            super::super::execution_admission::AdmittedCommitState,
        >,
        crate::transactions::data::CommitStructuralSummary,
        Vec<crate::validation::engine::InvariantExecutionResult>,
        crate::identity::data::VersionId,
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
        crate::authority::commit::phases::history::ResolvedCommitHistory,
        std::sync::Arc<crate::history::data::CanonicalCommitEnvelope>,
        crate::publication::patch::data::PatchStreamPosition,
        Vec<crate::transactions::data::RecordRef>,
        crate::snapshots::data::SnapshotHandle,
        Option<crate::mvcc::RelationalCoupledAuthoredSourceEffects>,
        Vec<crate::transactions::data::AspectEvaluationTrace>,
        Vec<crate::transactions::data::AspectEmissionTrace>,
    ) {
        (
            self.admitted,
            self.public_structural_summary,
            self.invariant_executions,
            self.version_id,
            self.created_entities,
            self.created_relations,
            self.history,
            self.canonical_commit_envelope,
            self.patch_position,
            self.changed_records,
            self.publication_snapshot,
            self.source_effects,
            self.aspect_evaluation_traces,
            self.aspect_emission_traces,
        )
    }
}
