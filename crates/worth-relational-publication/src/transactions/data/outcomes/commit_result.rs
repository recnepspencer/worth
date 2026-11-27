use branch::snapshots::data::SnapshotHandleFieldsPort as _;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
pub mod assembly;
pub mod seal;
use branch::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelope;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::runtime::state::subsystems::visibility::snapshot_handles::published_release::PublishedSnapshotReleaseObservation;
use branch::snapshots::data::SnapshotHandle;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact;
use contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use contracts::publication::bundle::publication_status::PublicationStatus;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch;
use contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion;
use contracts::schema::data::continuity::transition_plans::SchemaTransitionSummary;
use contracts::snapshots::data::coordinates::SnapshotId;
use contracts::transactions::data::aspect_reports::AspectTagAccuracyReport;
use contracts::transactions::data::aspect_reports::PatchVsTruthDeltaReport;
use contracts::transactions::data::aspect_traces::AspectEmissionTrace;
use contracts::transactions::data::aspect_traces::AspectEvaluationTrace;
use contracts::transactions::data::commit_log::CommitAspectSummary;
use contracts::transactions::data::commit_log::CommitChangeSummary;
use contracts::transactions::data::commit_log::CommitHistorySummary;
use contracts::transactions::data::commit_log::CommitLog;
use contracts::transactions::data::commit_log::CommitPatchBudgetSummary;
use contracts::transactions::data::commit_log::CommitPublicationSummary;
use contracts::transactions::data::commit_log::CommitSummary;
use contracts::transactions::data::primitives::RecordRef;
use contracts::transactions::data::primitives::TransactionId;
use std::ops::Deref;
use std::sync::Arc;

use contracts::transactions::data::outcomes::commit_validation::CommitValidationSummary;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use worth_relational_mutation::internal_port::transactions::data::outcomes::live_commit_validation::CommitValidation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitOutcome {
    pub transaction_id: TransactionId,
    pub commit: contracts::history::data::commit_receipt::RelationalCommitReceipt,
    pub version_id: contracts::identity::data::primitives::VersionId,
    pub snapshot: SnapshotHandle,
    pub changed_records: Vec<RecordRef>,
    pub publication_status: PublicationStatus,
    pub commit_log: CommitLog,
}

mod phase_timing;
pub use phase_timing::CommitPhaseTiming;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitPublication {
    pub(crate) source_effects: Option<
        crate::mvcc::publication::application_publication::RelationalCoupledAuthoredSourceEffects,
    >,
    pub diagnostics: Vec<RelationalDiagnosticArtifact>,
    pub envelope: SharedCanonicalCommitEnvelope,
    pub patch_position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
    pub aspect_evaluation_traces: Vec<AspectEvaluationTrace>,
    pub aspect_emission_traces: Vec<AspectEmissionTrace>,
    pub strategy_artifacts: Option<contracts::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle>,
}

pub use contracts::transactions::data::outcomes::commit_summaries::CommitSchemaSummary;
pub use contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitExecution {
    pub phase_timing: CommitPhaseTiming,
    pub complexity_delta: RuntimeComplexityCounters,
}

/// Immutable association produced by one authoritative commit execution.
///
/// Commit axes are observable through read-only methods, but cannot be swapped
/// across results after the authority owner seals them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitResult {
    snapshot_release: Arc<PublishedSnapshotReleaseObservation>,
    semantic_result: Arc<
        contracts::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult,
    >,
    outcome: CommitOutcome,
    summary: CommitSummary,
    structural_summary: CommitStructuralSummary,
    schema_summary: CommitSchemaSummary,
    publication: CommitPublication,
    validation: CommitValidation,
    execution: CommitExecution,
    created_entities: CommitCreatedEntityBindings,
    created_relations: CommitCreatedRelationBindings,
    late_interruption:
        Option<contracts::runtime::operation_control::interruption_event::RelationalInterruptionEvent>,
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
    pub(crate) fn snapshot_release_observation(&self) -> &Arc<PublishedSnapshotReleaseObservation> {
        &self.snapshot_release
    }
    /// The same immutable original semantic value sealed in this publication.
    pub fn semantic_result(
        &self,
    ) -> &contracts::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult
{
        &self.semantic_result
    }

    /// Seals one authoritative commit result together with the create-reference
    /// correspondence produced by that same commit execution.
    ///
    /// The commit pipeline is the sole minter of the sealed input. Every result
    /// axis remains private so no caller can detach or replace part of the
    /// authoritative association after construction.
    pub fn outcome(&self) -> &CommitOutcome {
        &self.outcome
    }

    pub const fn late_interruption(
        &self,
    ) -> Option<
        contracts::runtime::operation_control::interruption_event::RelationalInterruptionEvent,
    > {
        self.late_interruption
    }

    /// Resolves the record identity Relational assigned to this exact create
    /// reference while applying this commit.
    ///
    /// The correspondence stays inseparable from the `CommitResult`; callers
    /// cannot replace its map or rebuild a lookup from decomposed axes.
    pub fn created_entity(
        &self,
        created: &contracts::transactions::data::primitives::CreatedEntityRef,
    ) -> Option<contracts::identity::data::record_ids::EntityId> {
        self.created_entities.resolve(created)
    }

    /// Resolves the relation identity assigned to this exact create
    /// correspondence by the authoritative commit that produced this result.
    pub fn created_relation(
        &self,
        created: &contracts::transactions::data::primitives::CreatedRelationRef,
    ) -> Option<contracts::identity::data::record_ids::RelationId> {
        self.created_relations.resolve(created)
    }

    /// Returns every `(PersistentNameHolder, PersistentNameBirthRecord)` minted
    /// during THIS sealed commit, derived strictly from this commit's own
    /// canonical lineage events and record allocations in O(commit events) time
    /// with zero runtime lock acquisitions and immunity to concurrent post-commit
    /// retirements.
    pub fn committed_name_births(
        &self,
    ) -> Vec<(
        contracts::lineage::data::naming::PersistentNameHolder,
        contracts::lineage::data::naming::PersistentNameBirthRecord,
    )> {
        worth_relational_lineage::internal_port::lineage::authority::commit_births::extract_committed_name_births(&self.publication.envelope)
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
            touched_aspects:
                contracts::publication::patch::data::aspect_key_ordering::ordered_aspect_keys(
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

    pub fn invariant_executions(&self) -> &[worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult]{
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

    pub fn patch_position(
        &self,
    ) -> contracts::publication::patch::data::patch_position::PatchStreamPosition {
        self.publication.patch_position
    }

    pub fn final_snapshot_id(&self) -> SnapshotId {
        *self.outcome.snapshot.owner_snapshot_id()
    }

    pub fn merge_parent_count(&self) -> usize {
        self.outcome.commit.parents.len().saturating_sub(1)
    }
}

pub mod authored_source;
mod group_lineage;

#[path = "commit_result_owner_port.rs"]
mod owner_port;
pub use owner_port::CommitResultOwnerPort;

pub mod snapshot_fixture;
