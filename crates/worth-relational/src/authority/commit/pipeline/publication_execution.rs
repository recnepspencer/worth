use super::artifact_execution::AssembledCommitExecution;
use super::phase_backing::CommitPhaseBacking;

mod index_omission;
mod index_preparation;
mod phase;
mod preparation;
mod prepared_candidate;

use phase::finalize_publication_phase;
use preparation::{
    prepare_publication_phase, PreparePublicationPhaseInput, PreparedPublicationCompletion,
    PreparedPublicationPhase,
};

pub(crate) struct PublishedCommitExecution {
    admitted: CommitPhaseBacking<super::execution_admission::AdmittedCommitState>,
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

mod published;

/// Required request borrow proves this work belongs to the caller, without retaining it.
pub(super) fn prepare_commit_publication_execution(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    assembled: CommitPhaseBacking<AssembledCommitExecution>,
    required_output_publication: Option<crate::mvcc::RequiredOutputPublicationAdmission>,
    required_output_materialization: Option<
        std::sync::Arc<crate::history::data::CanonicalRequiredOutputMaterialization>,
    >,
    caller_association: Option<crate::mvcc::RelationalCallerAssociation>,
) -> Result<
    CommitPhaseBacking<PreparedCommitPublicationExecution>,
    crate::transactions::data::TransactionCommitError,
> {
    // Producer temporaries retire before funding the retained successor,
    // matching the former caller-side allocation boundary.
    let source_effects =
        required_output_publication
            .as_ref()
            .and_then(|admission| match &admission.publication {
                crate::mvcc::RequiredOutputPublication::CoupledAuthored { group } => {
                    Some(group.source_effects.clone())
                }
                _ => None,
            });
    let result = {
        let assembled = assembled.into_inner();
        super::consult_stage(execution, super::CommitStage::PublicationPreparation)?;
        let (
            mut admitted,
            selected_branch_state,
            public_structural_summary,
            working_state,
            invariant_executions,
            version_id,
            created_entities,
            created_relations,
            mutation_identities,
            history,
            merge_parent_branches,
            publication,
            publication_snapshot,
            aspect_evaluation_traces,
            aspect_emission_traces,
            source_interpretation,
            lineage_deferred,
        ) = assembled.into_parts();
        let predecessor_root = std::sync::Arc::clone(selected_branch_state.state().root());
        let publication = {
            let recovered_codec_publication_custody = admitted
                .validation_input()
                .recovered_codec_publication_custody()
                .cloned();
            let (_, _, _, _, commit_log, _) = admitted.phase_view().into_parts();
            prepare_publication_phase(
                execution,
                runtime,
                PreparePublicationPhaseInput {
                    lineage_deferred,
                    structural: &public_structural_summary,
                    validation: &invariant_executions,
                    caller_association,
                    source_interpretation: &source_interpretation,
                    recovered_codec_publication_custody,
                    required_output_publication,
                    required_output_materialization,
                    created_entities: &created_entities,
                    created_relations: &created_relations,
                    commit_log,
                    working_state,
                    mutation_identities,
                    selected_branch_state,
                    publication,
                    version_id,
                    previous_branch_head_version: history.previous_branch_head_version,
                    commit_id: history.commit_id,
                    commit_reference: history.commit_reference.clone(),
                    branch_basis: history.branch_basis.clone(),
                    branch_id: history.branch_id.clone(),
                    merge_base_commits: history.merge_base_commits.clone(),
                    merge_parent_branches,
                },
            )?
        };
        let prepared_indexes = index_preparation::prepare(
            execution,
            &runtime.index_definitions,
            runtime.commit_work_budget.as_ref(),
            &predecessor_root,
            publication.value().prepared_root(),
            admitted
                .validation_input()
                .home_target()
                .and_then(|target| target.index_definitions.as_deref()),
            admitted
                .validation_input()
                .home_step()
                .map(|step| step.index_budget),
        )
        .map_err(|error| {
            let (_, _, _, _, commit_log, _) = admitted.phase_view().into_parts();
            error.with_commit_log(commit_log.clone())
        })?;
        PreparedCommitPublicationExecution {
            source_interpretation,
            admitted: CommitPhaseBacking::new(execution, admitted)?,
            public_structural_summary,
            invariant_executions,
            version_id,
            created_entities,
            created_relations,
            history,
            publication,
            publication_snapshot,
            source_effects,
            aspect_evaluation_traces,
            aspect_emission_traces,
            prepared_indexes,
        }
    };
    CommitPhaseBacking::new(execution, result)
}

pub(crate) fn publish_commit_execution(
    runtime: &crate::runtime::RelationalRuntime,
    prepared: PreparedCommitPublicationCompletion,
    published_snapshot_basis: crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    positioned: std::sync::Arc<crate::history::data::PositionedCanonicalCommit>,
    envelope_cell: std::sync::Arc<crate::history::RelationalCommitEnvelopeResidency>,
) -> PublishedCommitExecution {
    let PreparedCommitPublicationCompletion {
        source_interpretation,
        mut admitted,
        public_structural_summary,
        invariant_executions,
        version_id,
        created_entities,
        created_relations,
        history,
        publication,
        publication_snapshot,
        source_effects,
        aspect_evaluation_traces,
        aspect_emission_traces,
        published_snapshot_slot,
        prepared_indexes,
    } = prepared;
    let finalized = {
        let (_, _, _, _, commit_log, phase_timing) = admitted.value_mut().phase_view().into_parts();
        finalize_publication_phase(
            runtime,
            commit_log,
            phase_timing,
            publication,
            published_snapshot_basis,
            published_snapshot_slot,
            positioned,
            envelope_cell,
        )
    };
    let (positioned_commit, changed_records) = finalized.into_parts();
    drop(source_interpretation);
    if let Some(prepared_indexes) = prepared_indexes {
        prepared_indexes.publish(runtime);
    }
    let patch_position = positioned_commit.position();
    let canonical_commit_envelope = std::sync::Arc::clone(positioned_commit.canonical_arc());
    let aspect_emission_traces: Vec<crate::transactions::data::AspectEmissionTrace> =
        aspect_emission_traces
            .into_iter()
            .map(|trace| trace.publish(patch_position))
            .collect();
    runtime.publish_invariant_preparation_diagnostics(&invariant_executions);
    publish_prepared_trace_diagnostics(runtime, &aspect_evaluation_traces, &aspect_emission_traces);
    if let Some(accounting) = admitted.value_mut().take_merge_accounting() {
        runtime.performance_access().count_merge_execution_request(
            accounting.admitted_records,
            accounting.emitted_mutation_intents,
        );
    }
    PublishedCommitExecution {
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
    }
}

fn publish_prepared_trace_diagnostics(
    runtime: &crate::runtime::RelationalRuntime,
    evaluation_traces: &[crate::transactions::data::AspectEvaluationTrace],
    emission_traces: &[crate::transactions::data::AspectEmissionTrace],
) {
    if !runtime.config.diagnostics.profile.detailed_traces_enabled {
        return;
    }
    for trace in evaluation_traces {
        runtime.push_preparation_diagnostic_artifact(trace.diagnostic_artifact());
    }
    for trace in emission_traces {
        runtime.push_preparation_diagnostic_artifact(trace.diagnostic_artifact());
    }
}

pub(crate) struct PreparedCommitPublicationExecution {
    source_interpretation:
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
    admitted: CommitPhaseBacking<super::execution_admission::AdmittedCommitState>,
    public_structural_summary: crate::transactions::data::CommitStructuralSummary,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
    version_id: crate::identity::data::VersionId,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    publication: CommitPhaseBacking<PreparedPublicationPhase>,
    publication_snapshot: crate::snapshots::data::SnapshotHandle,
    source_effects: Option<crate::mvcc::RelationalCoupledAuthoredSourceEffects>,
    aspect_evaluation_traces: Vec<crate::transactions::data::AspectEvaluationTrace>,
    aspect_emission_traces:
        Vec<super::artifact_execution::preparation::PreparedAspectEmissionTrace>,
    prepared_indexes: Option<crate::indexes::PreparedCandidateIndexPublication>,
}

pub(crate) struct PreparedCommitPublicationCompletion {
    pub(crate) source_interpretation:
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
    admitted: CommitPhaseBacking<super::execution_admission::AdmittedCommitState>,
    public_structural_summary: crate::transactions::data::CommitStructuralSummary,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
    version_id: crate::identity::data::VersionId,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    publication: PreparedPublicationCompletion,
    publication_snapshot: crate::snapshots::data::SnapshotHandle,
    source_effects: Option<crate::mvcc::RelationalCoupledAuthoredSourceEffects>,
    aspect_evaluation_traces: Vec<crate::transactions::data::AspectEvaluationTrace>,
    aspect_emission_traces:
        Vec<super::artifact_execution::preparation::PreparedAspectEmissionTrace>,
    published_snapshot_slot: crate::runtime::PublishedSnapshotSlotReservation,
    prepared_indexes: Option<crate::indexes::PreparedCandidateIndexPublication>,
}

impl PublishedCommitExecution {
    pub(crate) fn append_diagnostics(
        &mut self,
        diagnostics: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    ) {
        self.admitted.value_mut().append_diagnostics(diagnostics);
    }
}
#[cfg(test)]
#[path = "publication_execution_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

impl PreparedCommitPublicationCompletion {
    pub(crate) fn admit_prepared_indexes(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expires_at: std::time::Instant,
        maximum_lifetime_millis: u64,
    ) -> Result<(), crate::mvcc::RelationalPublicationOutcome> {
        if let Some(indexes) = &mut self.prepared_indexes {
            indexes.admit_catalog(execution, expires_at, maximum_lifetime_millis)?;
        }
        Ok(())
    }

    pub(crate) fn install_prepared_indexes(&mut self) {
        if let Some(indexes) = &mut self.prepared_indexes {
            indexes.install_admitted();
        }
    }
}
