use super::publication_execution::PublishedCommitExecution;
use crate::performance::operation_complexity_accounting::{
    combine_complexity_deltas, complexity_delta,
};
use crate::publication::bundle::PublicationStatus;
use crate::schema::data::SchemaTransitionSummary;
use crate::transactions::data::{
    CommitExecution, CommitOutcome, CommitPublication, CommitResult, CommitSchemaSummary,
    CommitValidation,
};

struct CommitResultMaterial {
    transaction_id: crate::transactions::data::TransactionId,
    phase_timing: crate::transactions::data::CommitPhaseTiming,
    commit_log: crate::transactions::data::CommitLog,
    strategy_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    diagnostics: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    complexity_before: crate::performance::data::RuntimeComplexityCounters,
    prior_complexity_delta: crate::performance::data::RuntimeComplexityCounters,
    public_structural_summary: crate::transactions::data::CommitStructuralSummary,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
    version_id: crate::identity::data::VersionId,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    canonical_commit_envelope: std::sync::Arc<crate::history::data::CanonicalCommitEnvelope>,
    patch_position: crate::publication::patch::data::PatchStreamPosition,
    changed_records: Vec<crate::transactions::data::RecordRef>,
    publication_snapshot: crate::snapshots::data::SnapshotHandle,
    source_effects: Option<crate::mvcc::RelationalCoupledAuthoredSourceEffects>,
    aspect_evaluation_traces: Vec<crate::transactions::data::AspectEvaluationTrace>,
    aspect_emission_traces: Vec<crate::transactions::data::AspectEmissionTrace>,
}

pub(crate) struct CommitResultSeal {
    outcome: CommitOutcome,
    summary: crate::transactions::data::CommitSummary,
    structural_summary: crate::transactions::data::CommitStructuralSummary,
    schema_summary: CommitSchemaSummary,
    publication: CommitPublication,
    validation: CommitValidation,
    execution: CommitExecution,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
}

impl CommitResultSeal {
    pub(crate) fn into_parts(
        self,
    ) -> (
        CommitOutcome,
        crate::transactions::data::CommitSummary,
        crate::transactions::data::CommitStructuralSummary,
        CommitSchemaSummary,
        CommitPublication,
        CommitValidation,
        CommitExecution,
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
        Option<crate::runtime::RelationalInterruptionEvent>,
    ) {
        (
            self.outcome,
            self.summary,
            self.structural_summary,
            self.schema_summary,
            self.publication,
            self.validation,
            self.execution,
            self.created_entities,
            self.created_relations,
            self.late_interruption,
        )
    }
}

pub(crate) fn assemble_commit_result(
    runtime: &crate::runtime::RelationalRuntime,
    published: PublishedCommitExecution,
    late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
) -> CommitResult {
    let material = CommitResultMaterial::from_published(published);
    let complexity_after = runtime.performance_access().complexity_counters_snapshot();
    let diagnostics = material.diagnostics.clone();
    material.assemble(diagnostics, complexity_after, late_interruption)
}

impl CommitResultMaterial {
    fn from_published(published: PublishedCommitExecution) -> Self {
        let (
            admitted,
            public_structural_summary,
            invariant_executions,
            version_id,
            created_entities,
            created_relations,
            history,
            canonical_commit_envelope,
            patch_position,
            changed_records,
            publication_snapshot,
            source_effects,
            aspect_evaluation_traces,
            aspect_emission_traces,
        ) = published.into_parts();
        let (
            transaction_id,
            phase_timing,
            commit_log,
            strategy_artifacts,
            diagnostics,
            complexity_before,
            prior_complexity_delta,
        ) = admitted.into_result_parts();
        Self {
            transaction_id,
            phase_timing,
            commit_log,
            strategy_artifacts,
            diagnostics,
            complexity_before,
            prior_complexity_delta,
            public_structural_summary,
            invariant_executions,
            version_id,
            created_entities,
            created_relations,
            history,
            canonical_commit_envelope,
            patch_position,
            changed_records,
            publication_snapshot,
            source_effects,
            aspect_evaluation_traces,
            aspect_emission_traces,
        }
    }

    fn assemble(
        self,
        diagnostics: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
        complexity_after: crate::performance::data::RuntimeComplexityCounters,
        late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
    ) -> CommitResult {
        let commit_summary = self.commit_log.summary().clone();
        let schema_summary = commit_schema_summary(&self.canonical_commit_envelope);
        let commit_outcome = CommitOutcome {
            transaction_id: self.transaction_id,
            commit: self.history.commit_reference,
            version_id: self.version_id,
            snapshot: self.publication_snapshot,
            changed_records: self.changed_records,
            publication_status: PublicationStatus::Published,
            commit_log: self.commit_log,
        };
        CommitResult::from_authoritative_commit(CommitResultSeal {
            outcome: commit_outcome,
            summary: commit_summary,
            structural_summary: self.public_structural_summary,
            schema_summary,
            publication: CommitPublication {
                diagnostics,
                source_effects: self.source_effects,
                envelope: self.canonical_commit_envelope,
                patch_position: self.patch_position,
                aspect_evaluation_traces: self.aspect_evaluation_traces,
                aspect_emission_traces: self.aspect_emission_traces,
                strategy_artifacts: self.strategy_artifacts,
            },
            validation: CommitValidation {
                summary: CommitValidation::summarize(&self.invariant_executions),
                invariant_executions: self.invariant_executions,
            },
            execution: CommitExecution {
                phase_timing: self.phase_timing,
                complexity_delta: combine_complexity_deltas(
                    self.prior_complexity_delta,
                    complexity_delta(self.complexity_before, complexity_after),
                ),
            },
            created_entities: self.created_entities,
            created_relations: self.created_relations,
            late_interruption,
        })
    }
}

fn commit_schema_summary(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
) -> CommitSchemaSummary {
    CommitSchemaSummary {
        transition: envelope
            .schema_transition
            .as_ref()
            .map(SchemaTransitionSummary::from_artifact),
        descriptor_semantics_version: envelope.descriptor_semantics_version,
    }
}
