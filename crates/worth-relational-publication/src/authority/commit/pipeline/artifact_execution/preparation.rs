use super::super::phase_backing::CommitPhaseBacking;
use worth_relational_branch::internal_port::snapshots::data::SnapshotHandleFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;
pub mod authoritative_publication;
use crate::authority::commit::phases::publication::canonicalize_changed_records;
use crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan;
use crate::authority::commit::publication::diagnostics_summary_artifact;
pub use authoritative_publication::ArtifactAuthority;
use authoritative_publication::{
    prepare_authoritative_publication, PreparedPublicationAuthority, PublicationAuthorityInput,
};
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::publication::patch::data::authoritative_patch_envelope::CanonicalAuthoritativePatch;
use contracts::transactions::data::aspect_traces::AspectEvaluationTrace;
use contracts::transactions::data::commit_log::CommitAspectSummary;
use contracts::transactions::data::commit_log::CommitChangeSummary;
use contracts::transactions::data::commit_log::CommitPublicationSummary;
use contracts::transactions::data::outcomes::plan_artifacts::PublishedMergeExecutionAuthority;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use worth_relational_mutation::internal_port::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
pub mod summary;
pub mod traces;
pub(in crate::internal_port::authority::commit::pipeline) use summary::summarize_commit_aspects;

pub(in crate::internal_port::authority::commit::pipeline) use traces::PreparedAspectEmissionTrace;
use traces::{capture_publication_traces, PublicationTraceCapture};

pub(in crate::internal_port::authority::commit::pipeline) struct PublicationPreparation {
    change_summary: CommitChangeSummary,
    aspect_summary: CommitAspectSummary,
    aspect_evaluation_traces: Vec<AspectEvaluationTrace>,
    aspect_emission_traces: Vec<PreparedAspectEmissionTrace>,
    summary: CommitPublicationSummary,
    finalize: PublicationFinalizeArtifacts,
}

pub(in crate::internal_port::authority::commit::pipeline) struct PublicationFinalizeArtifacts {
    artifacts: crate::publication::authority::artifacts::PublicationArtifacts,
    changed_records: Vec<RecordRef>,
    canonical_commit_envelope: worth_relational_branch::internal_port::history::data::canonical_commit_envelope::draft::CanonicalCommitEnvelopeDraft,
    adjacency_deltas: Vec<contracts::authority::mutation::adjacency_delta::AdjacencyDelta>,
    lineage_nodes: Vec<contracts::lineage::data::graph::LineageNode>,
    relation_lineages: Vec<(
        contracts::identity::data::record_ids::RelationId,
        contracts::identity::data::primitives::LineageId,
    )>,
    deferred_diagnostic_artifacts: Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
    target_schema_registry:
        Option<worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry>,
}

pub(super) struct PublicationPreparationInput<'a> {
    pub(super) defer_lineage: bool,
    pub(super) working_state: &'a mut worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    pub(super) content: worth_relational_branch::internal_port::content::publication::ContentPublication,
    pub(super) selected_branch_state: &'a worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    pub(super) patch: CanonicalAuthoritativePatch,
    pub(super) commit_reference: &'a RelationalCommitReceipt,
    pub(super) branch_id: &'a contracts::history::data::coordinates::BranchId,
    pub(super) version_id: contracts::identity::data::primitives::VersionId,
    pub(super) merge_parent_branches: &'a [contracts::history::data::coordinates::BranchId],
    pub(super) merge_base_commits: &'a [contracts::history::data::coordinates::CommitId],
    pub(super) merged_plan: RelationalSemanticPlan<'a>,
    pub(super) created_entities: &'a contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    pub(super) created_relations: &'a contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    pub(super) naming_plan: Option<&'a contracts::lineage::data::postures::CommitNamingPlan>,
    pub(super) record_allocations: &'a worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations,
    pub(super) strategy_artifacts:
        Option<contracts::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle>,
    pub(super) merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    pub(super) schema_continuity: &'a SchemaContinuityPlan,
    pub(super) effect: worth_relational_mutation::internal_port::authority::mutation::effect::MutationEffect,
    pub(super) additional_diagnostics_entries: Vec<RelationalDiagnosticsEntry>,
    pub(super) deferred_diagnostic_artifacts:
        Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
}

pub struct PublicationArtifactOwners<'a> {
    pub index_catalog: super::index_catalog::PublicationIndexCatalogInputs<'a>,
    pub authored_indexes: super::authored_index_writes::AuthoredIndexWriteInputs<'a>,
    pub authority: ArtifactAuthority<'a>,
    pub work: Option<&'a contracts::execution::request_work::RequestWorkBudget>,
}

pub(super) fn prepare_publication_artifacts(
    owners: PublicationArtifactOwners<'_>,
    input: PublicationPreparationInput<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<CommitPhaseBacking<PublicationPreparation>, TransactionCommitError> {
    let PublicationPreparationInput {
        defer_lineage,
        working_state,
        content,
        selected_branch_state,
        patch,
        commit_reference,
        branch_id,
        version_id,
        merge_parent_branches,
        merge_base_commits,
        merged_plan,
        created_entities,
        created_relations,
        naming_plan,
        record_allocations,
        strategy_artifacts,
        merge_execution_authority,
        schema_continuity,
        effect,
        additional_diagnostics_entries,
        deferred_diagnostic_artifacts,
    } = input;
    let worth_relational_mutation::internal_port::authority::mutation::effect::MutationEffect {
        publication,
        diagnostics,
        adjacency,
    } = effect;
    let (catalog, index_definition_delta) =
        super::index_catalog::prepare(owners.index_catalog, execution)?;
    let content_schema = match &schema_continuity.target_schema_registry {
        Some(registry) => registry.clone(),
        None => selected_branch_state
            .state()
            .root()
            .retained_schema_authority()
            .registry_arc(),
    };
    let content =
        contracts::execution::content_stage::run_commit_stage(execution, owners.work, || {
            content.prepare_final_fields(
                execution,
                selected_branch_state.state().root(),
                &content_schema,
                working_state,
                &publication.changed_records,
                version_id,
            )
        })
        .map_err(worth_relational_branch::internal_port::content::publication::commit_denial)?;
    let descriptive_touches = super::descriptive_touches::prepare(
        owners.work,
        owners.authority.envelope.symbols,
        &catalog,
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
        execution,
    )?;
    let authored_index_writes = super::authored_index_writes::prepare(
        owners.authored_indexes,
        &catalog,
        selected_branch_state,
        working_state,
        merged_plan,
        record_allocations.canonical(),
        schema_continuity.schema_transition.is_none()
            && merge_execution_authority.is_none()
            && merge_parent_branches.is_empty()
            && merge_base_commits.is_empty(),
        *execution,
    )?;
    let traces = capture_publication_traces(
        owners.authority.envelope.diagnostics.profile,
        &patch,
        &publication.canonical_deltas,
    );
    let diagnostics_summary = diagnostics_summary_artifact(
        owners
            .authority
            .envelope
            .diagnostics
            .profile
            .max_entries_per_artifact,
        additional_diagnostics_entries,
        diagnostics.entries,
    );
    let authority = prepare_authoritative_publication(
        execution,
        owners.authority,
        PublicationAuthorityInput {
            content,
            index_definition_delta,
            defer_lineage,
            working_state,
            patch: &patch,
            commit_reference,
            branch_id,
            version_id,
            merge_parent_branches,
            merge_base_commits,
            merged_plan,
            created_entities,
            created_relations,
            naming_plan,
            record_allocations,
            strategy_artifacts,
            merge_execution_authority,
            schema_continuity,
            changed_records: &publication.changed_records,
            diagnostics_summary: &diagnostics_summary,
            descriptive_touches,
            authored_index_writes,
        },
    )?;
    finish_publication_preparation(
        PublicationCompletionInput {
            patch,
            commit_reference,
            changed_records: publication.changed_records,
            canonical_deltas: publication.canonical_deltas,
            adjacency_deltas: adjacency.deltas,
            traces,
            authority,
            deferred_diagnostic_artifacts,
            target_schema_registry: schema_continuity.target_schema_registry.clone(),
        },
        execution,
    )
}

struct PublicationCompletionInput<'a> {
    patch: CanonicalAuthoritativePatch,
    commit_reference: &'a RelationalCommitReceipt,
    changed_records: Vec<RecordRef>,
    canonical_deltas: Vec<worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::data::CanonicalRecordAspectDelta>,
    adjacency_deltas: Vec<contracts::authority::mutation::adjacency_delta::AdjacencyDelta>,
    traces: PublicationTraceCapture,
    authority: CommitPhaseBacking<PreparedPublicationAuthority>,
    deferred_diagnostic_artifacts: Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
    target_schema_registry:
        Option<worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry>,
}

fn finish_publication_preparation(
    input: PublicationCompletionInput<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<CommitPhaseBacking<PublicationPreparation>, TransactionCommitError> {
    let result = {
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
        let authority = authority.into_inner();
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
            final_snapshot_id: Some(*authority.artifacts.snapshot.owner_snapshot_id()),
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
                relation_lineages: authority.relation_lineages,
                deferred_diagnostic_artifacts,
                target_schema_registry,
            },
        }
    };
    CommitPhaseBacking::new(execution, result)
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

    pub(in crate::internal_port::authority::commit::pipeline) fn snapshot(
        &self,
    ) -> &worth_relational_branch::internal_port::snapshots::data::SnapshotHandle {
        &self.finalize.artifacts.snapshot
    }

    pub(in crate::internal_port::authority::commit::pipeline) fn into_finalize(
        self,
    ) -> PublicationFinalizeArtifacts {
        self.finalize
    }
}

impl PublicationFinalizeArtifacts {
    pub(in crate::internal_port::authority::commit::pipeline) fn into_parts(
        self,
    ) -> (
        crate::publication::authority::artifacts::PublicationArtifacts,
        Vec<RecordRef>,
        worth_relational_branch::internal_port::history::data::canonical_commit_envelope::draft::CanonicalCommitEnvelopeDraft,
        Vec<contracts::authority::mutation::adjacency_delta::AdjacencyDelta>,
        Vec<contracts::lineage::data::graph::LineageNode>,
        Vec<(
            contracts::identity::data::record_ids::RelationId,
            contracts::identity::data::primitives::LineageId,
        )>,
        Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
        Option<worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry>,
    ){
        (
            self.artifacts,
            self.changed_records,
            self.canonical_commit_envelope,
            self.adjacency_deltas,
            self.lineage_nodes,
            self.relation_lineages,
            self.deferred_diagnostic_artifacts,
            self.target_schema_registry,
        )
    }
}
