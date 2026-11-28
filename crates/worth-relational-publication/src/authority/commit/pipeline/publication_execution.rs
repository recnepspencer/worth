use super::artifact_execution::AssembledCommitExecution;
use super::phase_backing::CommitPhaseBacking;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

pub mod index_omission;
pub mod index_preparation;
pub mod phase;
pub mod preparation;
pub mod prepared_candidate;
pub mod root_capture;
pub use root_capture::PublicationRootCapture;
pub mod phase_owners;
pub use phase_owners::PublicationPhaseOwners;

use phase::finalize_publication_phase;
use preparation::{
    prepare_publication_phase, PreparePublicationPhaseInput, PreparedPublicationCompletion,
    PreparedPublicationPhase,
};

pub struct PublishedCommitExecution {
    admitted: CommitPhaseBacking<super::execution_admission::AdmittedCommitState>,
    public_structural_summary: contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    invariant_executions: Vec<mutation::validation::engine::result::InvariantExecutionResult>,
    created_entities: contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    created_relations: contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    // Keep the admitted branch basis and work output alive through settlement diagnostics.
    _history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    _changed_records: Vec<contracts::transactions::data::primitives::RecordRef>,
    canonical_commit_envelope: std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>,
    patch_position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
    publication_snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    source_effects: Option<crate::mvcc::publication::application_publication::RelationalCoupledAuthoredSourceEffects>,
    aspect_evaluation_traces: Vec<contracts::transactions::data::aspect_traces::AspectEvaluationTrace>,
    aspect_emission_traces: Vec<contracts::transactions::data::aspect_traces::AspectEmissionTrace>,
}

pub mod published;

/// Required request borrow proves this work belongs to the caller, without retaining it.
pub(super) fn prepare_commit_publication_execution<
    Attach: FnOnce(
        &worth_relational_branch::internal_port::branch::root::PreparedRelationalBranchRootCapture,
    ),
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    owners: PublicationPhaseOwners<'_, Attach>,
    assembled: CommitPhaseBacking<AssembledCommitExecution>,
    required_output_publication: Option<
        crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
    >,
    required_output_materialization: Option<
        std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_materialization::CanonicalRequiredOutputMaterialization>,
    >,
    caller_association: Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
) -> Result<
    CommitPhaseBacking<PreparedCommitPublicationExecution>,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    // Producer temporaries retire before funding the retained successor,
    // matching the former caller-side allocation boundary.
    let source_effects =
        required_output_publication
            .as_ref()
            .and_then(|admission| match &admission.publication {
                crate::mvcc::publication::application_publication::RequiredOutputPublication::CoupledAuthored { group } => {
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
            publication,
            publication_snapshot,
            aspect_evaluation_traces,
            aspect_emission_traces,
            source_interpretation,
            lineage_deferred,
        ) = assembled.into_parts();
        let predecessor_root = std::sync::Arc::clone(selected_branch_state.state().root());
        let indexes = owners.indexes;
        let work = owners.work;
        let publication = {
            let recovered_codec_publication_custody = admitted
                .validation_input()
                .recovered_codec_publication_custody()
                .cloned();
            let (_, validation_input, _, commit_log, _) = admitted.phase_view().into_parts();
            let naming_plan = validation_input.naming_plan();
            prepare_publication_phase(
                execution,
                owners,
                PreparePublicationPhaseInput {
                    lineage_deferred,
                    naming_plan,
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
                },
            )?
        };
        let prepared_indexes = index_preparation::prepare(
            execution,
            indexes,
            work,
            &predecessor_root,
            publication.value().prepared_root(),
            admitted.validation_input().prepared_home_indexes.as_deref(),
            admitted
                .validation_input()
                .home_step()
                .map(|step| step.index_budget),
        )
        .map_err(|error| {
            let (_, _, _, commit_log, _) = admitted.phase_view().into_parts();
            error.with_commit_log(commit_log.clone())
        })?;
        PreparedCommitPublicationExecution {
            source_interpretation,
            admitted: CommitPhaseBacking::new(execution, admitted)?,
            public_structural_summary,
            invariant_executions,
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

pub mod settlement_inputs;
pub use settlement_inputs::PublicationSettlementInputs;

pub fn publish_commit_execution(
    inputs: PublicationSettlementInputs<'_>,
    publication: crate::publication::authority::SettlementPublication,
) -> PublishedCommitExecution {
    let (prepared, published_snapshot_basis, positioned, envelope_cell) = publication.into_parts();
    let PreparedCommitPublicationCompletion {
        source_interpretation,
        mut admitted,
        public_structural_summary,
        invariant_executions,
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
        let (_, _, _, commit_log, phase_timing) = admitted.value_mut().phase_view().into_parts();
        finalize_publication_phase(
            &inputs,
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
        prepared_indexes.publish();
    }
    let patch_position = positioned_commit.position();
    let canonical_commit_envelope = std::sync::Arc::clone(positioned_commit.canonical_arc());
    let aspect_emission_traces: Vec<
        contracts::transactions::data::aspect_traces::AspectEmissionTrace,
    > = aspect_emission_traces
        .into_iter()
        .map(|trace| trace.publish(patch_position))
        .collect();
    PublishedCommitExecution {
        admitted,
        public_structural_summary,
        invariant_executions,
        created_entities,
        created_relations,
        _history: history,
        _changed_records: changed_records,
        canonical_commit_envelope,
        patch_position,
        publication_snapshot,
        source_effects,
        aspect_evaluation_traces,
        aspect_emission_traces,
    }
}

pub struct PreparedCommitPublicationExecution {
    source_interpretation:
        std::sync::Arc<mutation::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
    admitted: CommitPhaseBacking<super::execution_admission::AdmittedCommitState>,
    public_structural_summary: contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    invariant_executions: Vec<mutation::validation::engine::result::InvariantExecutionResult>,
    created_entities: contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    created_relations: contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    publication: CommitPhaseBacking<PreparedPublicationPhase>,
    publication_snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    source_effects: Option<crate::mvcc::publication::application_publication::RelationalCoupledAuthoredSourceEffects>,
    aspect_evaluation_traces: Vec<contracts::transactions::data::aspect_traces::AspectEvaluationTrace>,
    aspect_emission_traces:
        Vec<super::artifact_execution::preparation::PreparedAspectEmissionTrace>,
    prepared_indexes: Option<worth_relational_indexes::internal_port::indexes::authority::maintenance::candidate_publication::PreparedCandidateIndexPublication>,
}

pub struct PreparedCommitPublicationCompletion {
    pub source_interpretation:
        std::sync::Arc<mutation::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
    admitted: CommitPhaseBacking<super::execution_admission::AdmittedCommitState>,
    public_structural_summary: contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    invariant_executions: Vec<mutation::validation::engine::result::InvariantExecutionResult>,
    created_entities: contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    created_relations: contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    publication: PreparedPublicationCompletion,
    publication_snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    source_effects: Option<crate::mvcc::publication::application_publication::RelationalCoupledAuthoredSourceEffects>,
    aspect_evaluation_traces: Vec<contracts::transactions::data::aspect_traces::AspectEvaluationTrace>,
    aspect_emission_traces:
        Vec<super::artifact_execution::preparation::PreparedAspectEmissionTrace>,
    published_snapshot_slot: worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
    prepared_indexes: Option<worth_relational_indexes::internal_port::indexes::authority::maintenance::candidate_publication::PreparedCandidateIndexPublication>,
}

#[cfg(test)]
#[path = "publication_execution_inline_type_budget.rs"]
pub mod inline_type_budget;

impl PreparedCommitPublicationCompletion {
    pub fn admit_prepared_indexes(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::mvcc::publication::outcome::RelationalPublicationOutcome> {
        if let Some(indexes) = &mut self.prepared_indexes {
            indexes.admit_catalog(execution).map_err(
                crate::mvcc::publication::outcome::RelationalPublicationOutcome::RequestStopped,
            )?;
        }
        Ok(())
    }

    pub(crate) fn install_prepared_indexes(&mut self) {
        if let Some(indexes) = &mut self.prepared_indexes {
            indexes.install_admitted();
        }
    }
}
