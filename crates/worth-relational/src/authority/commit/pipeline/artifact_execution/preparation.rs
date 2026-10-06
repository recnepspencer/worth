use crate::authority::commit::phases::publication::{
    canonical_commit_envelope, canonicalize_changed_records,
};
use crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan;
use crate::authority::commit::publication::diagnostics_summary_artifact;
use crate::diagnostics::data::RelationalDiagnosticsEntry;
use crate::diagnostics::data::{DiagnosticsArtifactKind, DiagnosticsScope};
use crate::history::data::RelationalCommitReceipt;
use crate::publication::patch::data::CanonicalAuthoritativePatch;
use crate::publication::{bundle::PublicationStage, data::PublicationError};
use crate::transactions::data::{
    AspectEvaluationTrace, CommitAspectSummary, CommitChangeSummary, CommitPublicationSummary,
    MergedCommitPlan, PublishedMergeExecutionAuthority, RecordRef, TransactionCommitError,
};
mod summary;
mod traces;
use summary::summarize_commit_aspects;

use traces::derive_aspect_emission_traces;
pub(in crate::authority::commit::pipeline) use traces::PreparedAspectEmissionTrace;

pub(in crate::authority::commit::pipeline) struct PublicationPreparation {
    change_summary: CommitChangeSummary,
    aspect_summary: CommitAspectSummary,
    aspect_evaluation_traces: Vec<AspectEvaluationTrace>,
    aspect_emission_traces: Vec<PreparedAspectEmissionTrace>,
    summary: CommitPublicationSummary,
    finalize: PublicationFinalizeArtifacts,
}

pub(in crate::authority::commit::pipeline) struct PublicationFinalizeArtifacts {
    artifacts: crate::storage::overlay::PublicationArtifacts,
    changed_records: Vec<RecordRef>,
    canonical_commit_envelope: crate::history::data::CanonicalCommitEnvelope,
    adjacency_deltas: Vec<crate::authority::mutation::AdjacencyDelta>,
    lineage_nodes: Vec<crate::lineage::data::LineageNode>,
    deferred_diagnostic_artifacts: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    target_schema_registry: Option<std::sync::Arc<crate::schema::data::RelationalSchemaRegistry>>,
}

pub(super) struct PublicationPreparationInput<'a> {
    pub(super) working_state: &'a mut crate::runtime::WorkingState,
    pub(super) selected_branch_state: &'a crate::branch::SelectedRelationalBranchState,
    pub(super) patch: CanonicalAuthoritativePatch,
    pub(super) commit_reference: &'a RelationalCommitReceipt,
    pub(super) branch_id: &'a crate::history::data::BranchId,
    pub(super) version_id: crate::identity::data::VersionId,
    pub(super) merge_parent_branches: &'a [crate::history::data::BranchId],
    pub(super) merge_base_commits: &'a [crate::history::data::CommitId],
    pub(super) merged_plan: &'a MergedCommitPlan,
    pub(super) record_allocations: &'a [crate::history::data::CanonicalRecordAllocation],
    pub(super) strategy_artifacts:
        Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    pub(super) merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    pub(super) schema_continuity: &'a SchemaContinuityPlan,
    pub(super) effect: crate::authority::mutation::MutationEffect,
    pub(super) additional_diagnostics_entries: Vec<RelationalDiagnosticsEntry>,
    pub(super) deferred_diagnostic_artifacts:
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
}

pub(super) fn prepare_publication_artifacts(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    input: PublicationPreparationInput<'_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<PublicationPreparation, TransactionCommitError> {
    let PublicationPreparationInput {
        working_state,
        selected_branch_state,
        patch,
        commit_reference,
        branch_id,
        version_id,
        merge_parent_branches,
        merge_base_commits,
        merged_plan,
        record_allocations,
        strategy_artifacts,
        merge_execution_authority,
        schema_continuity,
        effect,
        additional_diagnostics_entries,
        deferred_diagnostic_artifacts,
    } = input;
    let crate::authority::mutation::MutationEffect {
        publication,
        diagnostics,
        adjacency,
    } = effect;
    let descriptive_touches = super::descriptive_touches::prepare(
        runtime,
        selected_branch_state,
        working_state,
        &publication.canonical_deltas,
        &adjacency.deltas,
        if schema_continuity.schema_transition.is_some() {
            super::descriptive_touches::TouchPreparationMode::Unavailable
        } else if merge_execution_authority.is_some() {
            super::descriptive_touches::TouchPreparationMode::NativeMerge
        } else if !merge_parent_branches.is_empty() || !merge_base_commits.is_empty() {
            super::descriptive_touches::TouchPreparationMode::Unavailable
        } else {
            super::descriptive_touches::TouchPreparationMode::Ordinary
        },
        lease,
    )?;
    let traces = capture_publication_traces(runtime, &patch, &publication.canonical_deltas);
    let diagnostics_summary = diagnostics_summary_artifact(
        &runtime.config,
        additional_diagnostics_entries,
        diagnostics.entries,
    );
    let authority = prepare_authoritative_publication(
        runtime,
        PublicationAuthorityInput {
            working_state,
            patch: &patch,
            commit_reference,
            branch_id,
            version_id,
            merge_parent_branches,
            merge_base_commits,
            merged_plan,
            record_allocations,
            strategy_artifacts,
            merge_execution_authority,
            schema_continuity,
            changed_records: &publication.changed_records,
            diagnostics_summary: &diagnostics_summary,
            descriptive_touches,
        },
    )?;
    Ok(finish_publication_preparation(PublicationCompletionInput {
        patch,
        commit_reference,
        changed_records: publication.changed_records,
        canonical_deltas: publication.canonical_deltas,
        adjacency_deltas: adjacency.deltas,
        traces,
        authority,
        deferred_diagnostic_artifacts,
        target_schema_registry: schema_continuity.target_schema_registry.clone(),
    }))
}

struct PublicationTraceCapture {
    aspect_evaluation_traces: Vec<AspectEvaluationTrace>,
    aspect_emission_traces: Vec<PreparedAspectEmissionTrace>,
}

fn capture_publication_traces(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    patch: &CanonicalAuthoritativePatch,
    canonical_deltas: &[crate::authority::mutation::CanonicalRecordAspectDelta],
) -> PublicationTraceCapture {
    let diagnostics_profile = &runtime.config.diagnostics.profile;
    let capture_transaction_traces = diagnostics_profile.should_capture_artifact(
        DiagnosticsScope::Transaction,
        DiagnosticsArtifactKind::DetailedTrace,
    );
    let capture_patch_publication_traces = diagnostics_profile.should_capture_artifact(
        DiagnosticsScope::PatchPublication,
        DiagnosticsArtifactKind::DetailedTrace,
    );
    let aspect_evaluation_traces = if capture_transaction_traces {
        canonical_deltas
            .iter()
            .map(|delta| delta.evaluation_trace())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let aspect_emission_traces = if capture_patch_publication_traces {
        derive_aspect_emission_traces(&patch.authoritative_record_patches, canonical_deltas)
    } else {
        Vec::new()
    };
    PublicationTraceCapture {
        aspect_evaluation_traces,
        aspect_emission_traces,
    }
}

struct PublicationAuthorityInput<'a> {
    working_state: &'a mut crate::runtime::WorkingState,
    patch: &'a CanonicalAuthoritativePatch,
    commit_reference: &'a RelationalCommitReceipt,
    branch_id: &'a crate::history::data::BranchId,
    version_id: crate::identity::data::VersionId,
    merge_parent_branches: &'a [crate::history::data::BranchId],
    merge_base_commits: &'a [crate::history::data::CommitId],
    merged_plan: &'a MergedCommitPlan,
    record_allocations: &'a [crate::history::data::CanonicalRecordAllocation],
    strategy_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    schema_continuity: &'a SchemaContinuityPlan,
    changed_records: &'a [RecordRef],
    diagnostics_summary: &'a crate::diagnostics::data::RelationalDiagnosticArtifact,
    descriptive_touches: crate::history::data::RelationalDescriptiveTouchGraph,
}

struct PreparedPublicationAuthority {
    artifacts: crate::storage::overlay::PublicationArtifacts,
    canonical_commit_envelope: crate::history::data::CanonicalCommitEnvelope,
    lineage_event_count: usize,
    lineage_nodes: Vec<crate::lineage::data::LineageNode>,
}

fn prepare_authoritative_publication(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    input: PublicationAuthorityInput<'_>,
) -> Result<PreparedPublicationAuthority, TransactionCommitError> {
    let artifacts = runtime
        .publication_preparation_authority()
        .assemble_publication_bundle(
            input.commit_reference.clone(),
            input.version_id,
            input.patch.clone(),
            input.diagnostics_summary.clone(),
            input.schema_continuity.target_schema_authority().clone(),
        )
        .map_err(TransactionCommitError::publication_failed)?;
    let prepared_lineage = runtime
        .lineage_preparation_authority()
        .ensure_lineage_for_commit(
            input.working_state,
            input.commit_reference,
            &input.merged_plan.merged_intents,
            input.changed_records,
        )
        .map_err(|detail| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                detail,
            ))
        })?;
    let lineage_event_count = prepared_lineage
        .artifact()
        .event_batch()
        .counters()
        .event_batch_width;
    let (lineage_artifact, lineage_nodes) = prepared_lineage.into_parts();
    let mut canonical_commit_envelope = canonical_commit_envelope(
        runtime,
        input.commit_reference,
        input.branch_id,
        crate::history::data::CanonicalCommitAuthorityKind::VersionedTransaction,
        input.strategy_artifacts,
        input.merge_execution_authority,
        input.merge_parent_branches,
        input.merge_base_commits,
        input.merged_plan,
        input.patch.clone(),
        input.descriptive_touches,
        input.diagnostics_summary.clone(),
        lineage_artifact,
        crate::indexes::data::DerivedIndexArtifacts::default(),
        input.schema_continuity,
    )?;
    canonical_commit_envelope.install_record_allocations(input.record_allocations.to_vec());
    Ok(PreparedPublicationAuthority {
        artifacts,
        canonical_commit_envelope,
        lineage_event_count,
        lineage_nodes,
    })
}

struct PublicationCompletionInput<'a> {
    patch: CanonicalAuthoritativePatch,
    commit_reference: &'a RelationalCommitReceipt,
    changed_records: Vec<RecordRef>,
    canonical_deltas: Vec<crate::authority::mutation::CanonicalRecordAspectDelta>,
    adjacency_deltas: Vec<crate::authority::mutation::AdjacencyDelta>,
    traces: PublicationTraceCapture,
    authority: PreparedPublicationAuthority,
    deferred_diagnostic_artifacts: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    target_schema_registry: Option<std::sync::Arc<crate::schema::data::RelationalSchemaRegistry>>,
}

fn finish_publication_preparation(input: PublicationCompletionInput<'_>) -> PublicationPreparation {
    let PublicationCompletionInput {
        patch,
        commit_reference,
        changed_records,
        canonical_deltas,
        adjacency_deltas,
        traces,
        authority,
        deferred_diagnostic_artifacts,
        target_schema_registry,
    } = input;
    let mut changed_records = changed_records;
    canonicalize_changed_records(&mut changed_records);
    let change_summary = CommitChangeSummary {
        changed_record_count: changed_records.len(),
        adjacency_delta_count: adjacency_deltas.len(),
    };
    let aspect_summary = summarize_commit_aspects(&canonical_deltas);
    let summary = CommitPublicationSummary {
        patch_record_count: patch.authoritative_record_patches.len(),
        diagnostics_entry_count: authority.artifacts.diagnostics_summary.entries.len(),
        lineage_event_count: authority.lineage_event_count,
        patch_position: None,
        final_snapshot_id: Some(authority.artifacts.snapshot.snapshot_id),
        merge_parent_count: commit_reference.parents.len().saturating_sub(1),
    };

    PublicationPreparation {
        change_summary,
        aspect_summary,
        aspect_evaluation_traces: traces.aspect_evaluation_traces,
        aspect_emission_traces: traces.aspect_emission_traces,
        summary,
        finalize: PublicationFinalizeArtifacts {
            artifacts: authority.artifacts,
            changed_records,
            canonical_commit_envelope: authority.canonical_commit_envelope,
            adjacency_deltas,
            lineage_nodes: authority.lineage_nodes,
            deferred_diagnostic_artifacts,
            target_schema_registry,
        },
    }
}

impl PublicationPreparation {
    pub(super) fn summaries(
        &self,
    ) -> (
        &CommitChangeSummary,
        &CommitAspectSummary,
        &CommitPublicationSummary,
    ) {
        (&self.change_summary, &self.aspect_summary, &self.summary)
    }

    pub(super) fn aspect_evaluation_traces(&self) -> &[AspectEvaluationTrace] {
        &self.aspect_evaluation_traces
    }

    pub(super) fn aspect_emission_traces(&self) -> &[PreparedAspectEmissionTrace] {
        &self.aspect_emission_traces
    }

    pub(in crate::authority::commit::pipeline) fn snapshot(
        &self,
    ) -> &crate::snapshots::data::SnapshotHandle {
        &self.finalize.artifacts.snapshot
    }

    pub(in crate::authority::commit::pipeline) fn into_finalize(
        self,
    ) -> PublicationFinalizeArtifacts {
        self.finalize
    }
}

impl PublicationFinalizeArtifacts {
    pub(in crate::authority::commit::pipeline) fn into_parts(
        self,
    ) -> (
        crate::storage::overlay::PublicationArtifacts,
        Vec<RecordRef>,
        crate::history::data::CanonicalCommitEnvelope,
        Vec<crate::authority::mutation::AdjacencyDelta>,
        Vec<crate::lineage::data::LineageNode>,
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
        Option<std::sync::Arc<crate::schema::data::RelationalSchemaRegistry>>,
    ) {
        (
            self.artifacts,
            self.changed_records,
            self.canonical_commit_envelope,
            self.adjacency_deltas,
            self.lineage_nodes,
            self.deferred_diagnostic_artifacts,
            self.target_schema_registry,
        )
    }
}
