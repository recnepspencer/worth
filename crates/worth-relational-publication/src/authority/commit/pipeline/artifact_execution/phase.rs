use super::super::phase_backing::CommitPhaseBacking;
use super::super::rejection::{attach_rejection, elapsed_micros};
use super::preparation::{
    prepare_publication_artifacts, PublicationArtifactOwners, PublicationPreparation,
    PublicationPreparationInput,
};
use crate::authority::commit::phases::publication::enforce_patch_budget;
use crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan;
use crate::authority::commit::publication::assemble_patch;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use contracts::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::BranchId;
use contracts::identity::data::primitives::VersionId;
use contracts::transactions::data::commit_log::CommitLog;
use contracts::transactions::data::commit_log::CommitPatchBudgetSummary;
use contracts::transactions::data::commit_log::CommitPhase;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use worth_relational_mutation::internal_port::authority::mutation::effect::MutationEffect;

pub(super) struct ArtifactAssemblyInput<'a> {
    pub(super) defer_lineage: bool,
    pub(super) working_state: &'a mut worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    pub(super) content: worth_relational_branch::internal_port::content::publication::ContentPublication,
    pub(super) selected_branch_state: &'a worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    pub(super) effect: MutationEffect,
    pub(super) commit_reference: &'a RelationalCommitReceipt,
    pub(super) branch_id: &'a BranchId,
    pub(super) version_id: VersionId,
    pub(super) merged_plan: RelationalSemanticPlan<'a>,
    pub(super) created_entities: &'a contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    pub(super) created_relations: &'a contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    pub(super) naming_plan: Option<&'a contracts::lineage::data::postures::CommitNamingPlan>,
    pub(super) record_allocations: &'a worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations,
    pub(super) strategy_commit_artifacts: Option<StrategyCommitArtifactBundle>,
    pub(super) schema_continuity: &'a SchemaContinuityPlan,
    pub(super) deferred_diagnostic_artifacts:
        Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
}

pub(super) fn assemble_authoritative_publication_phase(
    owners: PublicationArtifactOwners<'_>,
    max_patch_records: Option<usize>,
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    input: ArtifactAssemblyInput<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<CommitPhaseBacking<PublicationPreparation>, TransactionCommitError> {
    commit_log.begin_phase(CommitPhase::ArtifactAssembly);
    let phase_started = worth_execution::MonotonicClock::system().now();
    let mut effect = input.effect;
    let patch_fragments = std::mem::take(&mut effect.publication.patch_fragments);
    let lease = contracts::execution::request_custody::request_lease(execution);
    let patch = assemble_patch(
        owners.authority.envelope.diagnostics.instrumentation,
        owners.work,
        patch_fragments,
        lease,
    )?;
    let patch_budget_summary = CommitPatchBudgetSummary {
        patch_record_count: patch.authoritative_record_patches.len(),
        max_patch_records_per_commit: max_patch_records,
    };
    commit_log.record_patch_budget(&patch_budget_summary);
    enforce_patch_budget(
        max_patch_records,
        owners.authority.envelope.diagnostics,
        &patch,
    )
    .map_err(|error| attach_rejection(commit_log, CommitPhase::ArtifactAssembly, error))?;
    let publication = prepare_publication_artifacts(
        owners,
        PublicationPreparationInput {
            defer_lineage: input.defer_lineage,
            working_state: input.working_state,
            content: input.content,
            selected_branch_state: input.selected_branch_state,
            patch,
            commit_reference: input.commit_reference,
            branch_id: input.branch_id,
            version_id: input.version_id,
            merged_plan: input.merged_plan,
            created_entities: input.created_entities,
            created_relations: input.created_relations,
            naming_plan: input.naming_plan,
            record_allocations: input.record_allocations,
            strategy_artifacts: input.strategy_commit_artifacts,
            schema_continuity: input.schema_continuity,
            effect,
            deferred_diagnostic_artifacts: input.deferred_diagnostic_artifacts,
        },
        execution,
    )
    .map_err(|error| attach_rejection(commit_log, CommitPhase::ArtifactAssembly, error))?;
    record_publication_phase_artifacts(commit_log, publication.value());
    commit_log.complete_phase(CommitPhase::ArtifactAssembly);
    phase_timing.artifact_assembly_micros = elapsed_micros(phase_started);
    Ok(publication)
}

fn record_publication_phase_artifacts(
    commit_log: &mut CommitLog,
    publication: &PublicationPreparation,
) {
    let (change_summary, aspect_summary, publication_summary) = publication.summaries();
    commit_log.record_changed_records(change_summary);
    commit_log.record_aspect_summary(aspect_summary);
    commit_log.record_publication_artifacts(publication_summary);
}
