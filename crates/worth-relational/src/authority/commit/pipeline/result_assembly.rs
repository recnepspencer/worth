use super::publication_execution::PublishedCommitExecution;
use crate::performance::operation_complexity_accounting::{
    combine_complexity_deltas, complexity_delta,
};
use crate::publication::bundle::PublicationStatus;
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
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    canonical_commit_envelope: std::sync::Arc<crate::history::data::CanonicalCommitEnvelope>,
    patch_position: crate::publication::patch::data::PatchStreamPosition,
    publication_snapshot: crate::snapshots::data::SnapshotHandle,
    source_effects: Option<crate::mvcc::RelationalCoupledAuthoredSourceEffects>,
    aspect_evaluation_traces: Vec<crate::transactions::data::AspectEvaluationTrace>,
    aspect_emission_traces: Vec<crate::transactions::data::AspectEmissionTrace>,
}

pub(crate) struct CommitResultSeal {
    snapshot_release: std::sync::Arc<crate::runtime::PublishedSnapshotReleaseObservation>,
    semantic_result: std::sync::Arc<crate::transactions::data::RelationalCommitSemanticResult>,
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
        std::sync::Arc<crate::runtime::PublishedSnapshotReleaseObservation>,
        std::sync::Arc<crate::transactions::data::RelationalCommitSemanticResult>,
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
            self.snapshot_release,
            self.semantic_result,
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
    let snapshot_release = runtime.visibility.published_release_observation(material.publication_snapshot.snapshot_id()).expect(/* Proof: PublishedCommitExecution carries the just-installed original snapshot. Its handle is private until this sealed result returns; no external caller can release it before assembly. */ "original published snapshot is registered before result sealing");
    let diagnostics = material.diagnostics.clone();
    material.assemble(
        diagnostics,
        complexity_after,
        late_interruption,
        snapshot_release,
    )
}

impl CommitResultMaterial {
    fn from_published(published: PublishedCommitExecution) -> Self {
        let (
            admitted,
            public_structural_summary,
            invariant_executions,
            _version_id,
            created_entities,
            created_relations,
            _history,
            canonical_commit_envelope,
            patch_position,
            _changed_records,
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
        ) = admitted.into_inner().into_result_parts();
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
            created_entities,
            created_relations,
            canonical_commit_envelope,
            patch_position,
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
        snapshot_release: std::sync::Arc<crate::runtime::PublishedSnapshotReleaseObservation>,
    ) -> CommitResult {
        let semantic_result = self.canonical_commit_envelope.semantic_result_arc();
        let structural = semantic_result.structural();
        let mut structural_summary = self.public_structural_summary;
        structural_summary.invariant_groups = structural.invariant_groups;
        structural_summary.commit_topology = structural.commit_topology;
        structural_summary.touched_partitions = structural.touched_partitions.clone();
        structural_summary.descriptor_semantics_version = structural.descriptor_semantics_version;
        let mut commit_summary = self.commit_log.summary().clone();
        commit_summary.structural_summary = Some(structural_summary.clone());
        commit_summary.history_summary = Some(semantic_result.history().clone());
        commit_summary.change_summary = Some(semantic_result.change().clone());
        commit_summary.aspect_summary = Some(semantic_result.aspects().clone());
        if let Some(patch) = &mut commit_summary.patch_budget_summary {
            patch.patch_record_count = semantic_result.publication().patch_record_count;
        }
        if let Some(publication) = &mut commit_summary.publication_summary {
            publication.patch_record_count = semantic_result.publication().patch_record_count;
            publication.lineage_event_count = semantic_result.publication().lineage_event_count;
            publication.merge_parent_count = semantic_result.publication().merge_parent_count;
        }
        let validation_summary = semantic_result.validation_summary();
        commit_summary.invariant_result_count = validation_summary.result_count;
        commit_summary.invariant_advisory_count = validation_summary.advisory_count;
        commit_summary.invariant_violation_count = validation_summary.violation_count;
        let schema_summary = semantic_result.schema().clone();
        let commit_outcome = CommitOutcome {
            transaction_id: self.transaction_id,
            commit: semantic_result.receipt().clone(),
            version_id: semantic_result.receipt().version_id,
            snapshot: self.publication_snapshot,
            changed_records: semantic_result.changed_records().to_vec(),
            publication_status: PublicationStatus::Published,
            commit_log: self.commit_log,
        };
        CommitResult::from_authoritative_commit(CommitResultSeal {
            snapshot_release,
            semantic_result,
            outcome: commit_outcome,
            summary: commit_summary,
            structural_summary,
            schema_summary,
            publication: CommitPublication {
                diagnostics,
                source_effects: self.source_effects,
                envelope: crate::history::data::SharedCanonicalCommitEnvelope::from_arc(
                    self.canonical_commit_envelope,
                ),
                patch_position: self.patch_position,
                aspect_evaluation_traces: self.aspect_evaluation_traces,
                aspect_emission_traces: self.aspect_emission_traces,
                strategy_artifacts: self.strategy_artifacts,
            },
            validation: CommitValidation {
                summary: validation_summary,
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
