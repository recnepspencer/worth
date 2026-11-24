use crate::authority::commit::pipeline::PublishedCommitExecution;
use crate::performance::operation_complexity_accounting::{
    combine_complexity_deltas, complexity_delta,
};
use crate::publication::bundle::PublicationStatus;
use crate::runtime::state::subsystems::visibility::PublishedSnapshotReleaseObservation;
use crate::transactions::data::outcomes::commit_result::CommitExecution;
use crate::transactions::data::outcomes::commit_result::CommitOutcome;
use crate::transactions::data::outcomes::commit_result::CommitPublication;
use crate::transactions::data::outcomes::commit_result::CommitResult;
use crate::transactions::data::outcomes::commit_summaries::CommitSchemaSummary;
use crate::transactions::data::outcomes::live_commit_validation::CommitValidation;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;

pub(crate) struct CommitResultSeal {
    snapshot_release: std::sync::Arc<PublishedSnapshotReleaseObservation>,
    semantic_result: std::sync::Arc<crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult>,
    outcome: CommitOutcome,
    summary: crate::transactions::data::commit_log::CommitSummary,
    structural_summary: crate::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    schema_summary: CommitSchemaSummary,
    publication: CommitPublication,
    validation: CommitValidation,
    execution: CommitExecution,
    created_entities: crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    late_interruption: Option<crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent>,
}

impl CommitResultSeal {
    pub(crate) fn into_parts(
        self,
    ) -> (
        std::sync::Arc<PublishedSnapshotReleaseObservation>,
        std::sync::Arc<crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult>,
        CommitOutcome,
        crate::transactions::data::commit_log::CommitSummary,
        crate::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
        CommitSchemaSummary,
        CommitPublication,
        CommitValidation,
        CommitExecution,
        crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        crate::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
        Option<crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent>,
    ){
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
    published: PublishedCommitExecution,
    complexity_after: crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
    late_interruption: Option<
        crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent,
    >,
    snapshot_release: std::sync::Arc<PublishedSnapshotReleaseObservation>,
) -> CommitResult {
    let (
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
    ) = published.into_result_operands();
    let semantic_result = canonical_commit_envelope.semantic_result_arc();
    let structural = semantic_result.structural();
    let mut structural_summary = public_structural_summary;
    structural_summary.invariant_groups = structural.invariant_groups;
    structural_summary.commit_topology = structural.commit_topology;
    structural_summary.touched_partitions = structural.touched_partitions.clone();
    structural_summary.descriptor_semantics_version = structural.descriptor_semantics_version;
    let mut commit_summary = commit_log.summary().clone();
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
        transaction_id,
        commit: semantic_result.receipt().clone(),
        version_id: semantic_result.receipt().version_id,
        snapshot: publication_snapshot,
        changed_records: semantic_result.changed_records().to_vec(),
        publication_status: PublicationStatus::Published,
        commit_log,
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
            source_effects,
            envelope: crate::history::data::SharedCanonicalCommitEnvelope::from_arc(
                canonical_commit_envelope,
            ),
            patch_position,
            aspect_evaluation_traces,
            aspect_emission_traces,
            strategy_artifacts,
        },
        validation: CommitValidation {
            summary: validation_summary,
            invariant_executions,
        },
        execution: CommitExecution {
            phase_timing,
            complexity_delta: combine_complexity_deltas(
                prior_complexity_delta,
                complexity_delta(complexity_before, complexity_after),
            ),
        },
        created_entities,
        created_relations,
        late_interruption,
    })
}
