use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::history::data::CanonicalCommitEnvelope;
use crate::performance::data::RuntimeComplexityCounters;
use crate::publication::bundle::PublicationStatus;
use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;
use crate::schema::data::{DescriptorSemanticsVersion, SchemaTransitionSummary};
use crate::snapshots::data::{SnapshotHandle, SnapshotId};
use crate::transactions::data::{
    AspectEmissionTrace, AspectEvaluationTrace, AspectTagAccuracyReport, CommitAspectSummary,
    CommitChangeSummary, CommitHistorySummary, CommitLog, CommitPatchBudgetSummary,
    CommitPublicationSummary, CommitSummary, PatchVsTruthDeltaReport, RecordRef, TransactionId,
};
use crate::validation::data::InvariantGroupSet;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Arc;

use super::{
    CommitCreatedEntityBindings, CommitCreatedRelationBindings, CommitValidation,
    CommitValidationSummary,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitOutcome {
    pub transaction_id: TransactionId,
    pub commit: crate::history::data::RelationalCommitReceipt,
    pub version_id: crate::identity::data::VersionId,
    pub snapshot: SnapshotHandle,
    pub changed_records: Vec<RecordRef>,
    pub publication_status: PublicationStatus,
    pub commit_log: CommitLog,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommitPhaseTiming {
    pub draft_preparation_micros: u64,
    pub draft_intent_normalization_micros: u64,
    pub draft_bulk_admission_micros: u64,
    pub draft_merge_plan_micros: u64,
    pub draft_intent_validation_micros: u64,
    pub draft_intent_sort_micros: u64,
    pub draft_conflict_detection_micros: u64,
    pub draft_structural_summary_micros: u64,
    pub draft_working_state_clone_micros: u64,
    pub working_state_preparation_micros: u64,
    pub invariant_pre_check_micros: u64,
    pub authoritative_mutation_micros: u64,
    pub history_resolution_micros: u64,
    pub invariant_post_check_micros: u64,
    pub artifact_assembly_micros: u64,
    pub durable_append_micros: u64,
    pub publication_micros: u64,
    pub publication_storage_commit_micros: u64,
    pub publication_index_refresh_micros: u64,
    pub publication_history_publish_micros: u64,
    pub publication_visibility_pin_micros: u64,
    pub publication_retention_reconcile_micros: u64,
    pub publication_compaction_micros: u64,
    pub publication_bundle_publish_micros: u64,
    pub publication_retention_pass_micros: u64,
    pub publication_post_commit_consumer_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitPublication {
    pub diagnostics: Vec<RelationalDiagnosticArtifact>,
    pub envelope: Arc<CanonicalCommitEnvelope>,
    pub patch_position: crate::publication::patch::data::PatchStreamPosition,
    pub aspect_evaluation_traces: Vec<AspectEvaluationTrace>,
    pub aspect_emission_traces: Vec<AspectEmissionTrace>,
    pub strategy_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitStructuralSummary {
    pub invariant_groups: InvariantGroupSet,
    pub commit_topology: crate::transactions::data::CommitTopology,
    pub touched_partitions: Vec<crate::identity::data::PartitionId>,
    pub bulk_entity_slots_reserved: usize,
    pub bulk_relation_slots_reserved: usize,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitSchemaSummary {
    pub transition: Option<SchemaTransitionSummary>,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitExecution {
    pub phase_timing: CommitPhaseTiming,
    pub complexity_delta: RuntimeComplexityCounters,
}

/// Immutable association produced by one authoritative commit execution.
///
/// Commit axes are observable through read-only methods, but cannot be swapped
/// across results after the authority owner seals them.
///
/// ```compile_fail
/// use worth_relational::facade::transactions::CommitResult;
///
/// fn cannot_swap_outcomes(target: &mut CommitResult, other: &CommitResult) {
///     target.outcome = other.outcome().clone();
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitResult {
    outcome: CommitOutcome,
    summary: CommitSummary,
    structural_summary: CommitStructuralSummary,
    schema_summary: CommitSchemaSummary,
    publication: CommitPublication,
    validation: CommitValidation,
    execution: CommitExecution,
    created_entities: CommitCreatedEntityBindings,
    created_relations: CommitCreatedRelationBindings,
    late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
}

impl Deref for CommitResult {
    type Target = CommitOutcome;

    fn deref(&self) -> &Self::Target {
        &self.outcome
    }
}

impl CommitOutcome {
    pub fn summary(&self) -> &CommitLog {
        &self.commit_log
    }

    pub fn commit_summary(&self) -> &CommitSummary {
        self.commit_log.summary()
    }

    pub fn history_summary(&self) -> Option<&CommitHistorySummary> {
        self.commit_summary().history_summary.as_ref()
    }

    pub fn patch_budget_summary(&self) -> Option<&CommitPatchBudgetSummary> {
        self.commit_summary().patch_budget_summary.as_ref()
    }

    pub fn change_summary(&self) -> Option<&CommitChangeSummary> {
        self.commit_summary().change_summary.as_ref()
    }

    pub fn publication_summary(&self) -> Option<&CommitPublicationSummary> {
        self.commit_summary().publication_summary.as_ref()
    }

    pub fn aspect_summary(&self) -> Option<&CommitAspectSummary> {
        self.commit_summary().aspect_summary.as_ref()
    }
}

impl CommitResult {
    /// Seals one authoritative commit result together with the create-reference
    /// correspondence produced by that same commit execution.
    ///
    /// The commit pipeline is the sole minter of the sealed input. Every result
    /// axis remains private so no caller can detach or replace part of the
    /// authoritative association after construction.
    pub(crate) fn from_authoritative_commit(
        seal: crate::authority::commit::CommitResultSeal,
    ) -> Self {
        let (
            outcome,
            summary,
            structural_summary,
            schema_summary,
            publication,
            validation,
            execution,
            created_entities,
            created_relations,
            late_interruption,
        ) = seal.into_parts();
        Self {
            outcome,
            summary,
            structural_summary,
            schema_summary,
            publication,
            validation,
            execution,
            created_entities,
            created_relations,
            late_interruption,
        }
    }

    pub fn outcome(&self) -> &CommitOutcome {
        &self.outcome
    }

    pub const fn late_interruption(&self) -> Option<crate::runtime::RelationalInterruptionEvent> {
        self.late_interruption
    }

    /// Resolves the record identity Relational assigned to this exact create
    /// reference while applying this commit.
    ///
    /// The correspondence stays inseparable from the `CommitResult`; callers
    /// cannot replace its map or rebuild a lookup from decomposed axes.
    pub fn created_entity(
        &self,
        created: &crate::transactions::data::CreatedEntityRef,
    ) -> Option<crate::identity::data::EntityId> {
        self.created_entities.resolve(created)
    }

    /// Resolves the relation identity assigned to this exact create
    /// correspondence by the authoritative commit that produced this result.
    pub fn created_relation(
        &self,
        created: &crate::transactions::data::CreatedRelationRef,
    ) -> Option<crate::identity::data::RelationId> {
        self.created_relations.resolve(created)
    }

    pub fn commit_log(&self) -> &CommitLog {
        &self.outcome.commit_log
    }

    pub fn commit_summary(&self) -> &CommitSummary {
        &self.summary
    }

    pub fn publication(&self) -> &CommitPublication {
        &self.publication
    }

    pub fn structural_summary(&self) -> &CommitStructuralSummary {
        &self.structural_summary
    }

    pub fn history_summary(&self) -> Option<&CommitHistorySummary> {
        self.summary.history_summary.as_ref()
    }

    pub fn patch_budget_summary(&self) -> Option<&CommitPatchBudgetSummary> {
        self.summary.patch_budget_summary.as_ref()
    }

    pub fn change_summary(&self) -> Option<&CommitChangeSummary> {
        self.summary.change_summary.as_ref()
    }

    pub fn publication_summary(&self) -> Option<&CommitPublicationSummary> {
        self.summary.publication_summary.as_ref()
    }

    pub fn aspect_summary(&self) -> Option<&CommitAspectSummary> {
        self.summary.aspect_summary.as_ref()
    }

    pub fn validation(&self) -> &CommitValidation {
        &self.validation
    }

    pub fn schema_summary(&self) -> &CommitSchemaSummary {
        &self.schema_summary
    }

    pub fn execution(&self) -> &CommitExecution {
        &self.execution
    }

    pub fn diagnostics(&self) -> &[RelationalDiagnosticArtifact] {
        &self.publication.diagnostics
    }

    pub fn aspect_evaluation_traces(&self) -> &[AspectEvaluationTrace] {
        &self.publication.aspect_evaluation_traces
    }

    pub fn aspect_emission_traces(&self) -> &[AspectEmissionTrace] {
        &self.publication.aspect_emission_traces
    }

    pub fn patch_vs_truth_delta_report(&self) -> PatchVsTruthDeltaReport {
        let patch = self.patch();
        let traces = self.aspect_emission_traces();
        let records_checked = patch.len().min(traces.len());
        let mut mismatched_targets = Vec::new();
        let mut structural_mismatches = 0;
        let mut aspect_mismatches = 0;
        let mut opaque_aspect_mismatches = 0;

        for (record, trace) in patch.iter().zip(traces.iter()) {
            if record.target != trace.target {
                mismatched_targets.push(record.target.clone());
            }
            if record.structural_change != trace.structural_change {
                structural_mismatches += 1;
            }
            if record.authoritative_changed_aspects() != trace.changed_aspects {
                aspect_mismatches += 1;
            }
            if record.contains_opaque_aspect != trace.contains_opaque_aspect {
                opaque_aspect_mismatches += 1;
            }
        }

        PatchVsTruthDeltaReport {
            records_checked: records_checked as u64,
            exact_match: mismatched_targets.is_empty()
                && structural_mismatches == 0
                && aspect_mismatches == 0
                && opaque_aspect_mismatches == 0
                && patch.len() == traces.len(),
            mismatched_targets,
            structural_mismatches: structural_mismatches as u64,
            aspect_mismatches: aspect_mismatches as u64,
            opaque_aspect_mismatches: opaque_aspect_mismatches as u64,
        }
    }

    pub fn aspect_tag_accuracy_report(&self) -> AspectTagAccuracyReport {
        let traces = self.aspect_emission_traces();
        AspectTagAccuracyReport {
            records_checked: traces.len() as u64,
            correctly_tagged_records: if self.patch_vs_truth_delta_report().exact_match {
                traces.len() as u64
            } else {
                traces
                    .iter()
                    .zip(self.patch().iter())
                    .filter(|(trace, record)| {
                        trace.changed_aspects == record.authoritative_changed_aspects()
                            && trace.contains_opaque_aspect == record.contains_opaque_aspect
                    })
                    .count() as u64
            },
            touched_aspects: crate::publication::patch::data::ordered_aspect_keys(
                traces
                    .iter()
                    .flat_map(|trace| trace.changed_aspects.iter().cloned()),
            ),
            opaque_aspect_record_count: traces
                .iter()
                .filter(|trace| trace.contains_opaque_aspect)
                .count() as u64,
        }
    }

    pub fn patch(&self) -> &[PublishedAuthoritativeRecordPatch] {
        &self.publication.envelope.patch.authoritative_record_patches
    }

    pub fn envelope(&self) -> &CanonicalCommitEnvelope {
        self.publication.envelope.as_ref()
    }

    pub fn schema_transition_summary(&self) -> Option<&SchemaTransitionSummary> {
        self.schema_summary.transition.as_ref()
    }

    pub fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion {
        self.schema_summary.descriptor_semantics_version
    }

    pub fn invariant_executions(&self) -> &[crate::validation::engine::InvariantExecutionResult] {
        self.validation.invariant_executions()
    }

    pub fn validation_summary(&self) -> CommitValidationSummary {
        self.validation.summary()
    }

    pub fn phase_timing(&self) -> &CommitPhaseTiming {
        &self.execution.phase_timing
    }

    pub fn complexity_delta(&self) -> &RuntimeComplexityCounters {
        &self.execution.complexity_delta
    }

    pub fn patch_position(&self) -> crate::publication::patch::data::PatchStreamPosition {
        self.publication.patch_position
    }

    pub fn final_snapshot_id(&self) -> SnapshotId {
        self.outcome.snapshot.snapshot_id
    }

    pub fn merge_parent_count(&self) -> usize {
        self.outcome.commit.parents.len().saturating_sub(1)
    }
}
