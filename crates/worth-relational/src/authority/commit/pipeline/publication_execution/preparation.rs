mod preparation_denial;
use std::sync::Arc;

use super::super::artifact_execution::preparation::PublicationPreparation;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use crate::publication::bundle::PublicationStage;
use crate::publication::data::PublicationError;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{CommitLog, TransactionCommitError};

pub(super) struct PreparedPublicationPhase {
    pub(super) mutation_identities:
        crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
    pub(super) mirror_publication: crate::storage::PreparedMirrorPublication,
    pub(super) prepared_history: crate::mvcc::PreparedRelationalPublication,
    pub(super) validated_lineage_events: crate::runtime::ValidatedLineageEventBatch,
    pub(super) lineage_nodes: Vec<crate::lineage::data::LineageNode>,
    pub(super) artifacts: crate::storage::overlay::PublicationArtifacts,
    pub(super) changed_records: Vec<crate::transactions::data::RecordRef>,
    pub(super) version_id: VersionId,
    pub(super) previous_branch_head_version: Option<VersionId>,
    pub(super) commit_id: CommitId,
    pub(super) commit_reference: RelationalCommitReceipt,
    pub(super) branch_id: BranchId,
    pub(super) merge_base_commits: Vec<CommitId>,
    pub(super) merge_parent_branches: Vec<BranchId>,
    pub(super) deferred_diagnostic_artifacts:
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    pub(super) canonical_publication_route: crate::runtime::PreparedCanonicalPublicationRoute,
}

pub(super) struct PreparedPublicationCompletion {
    pub(super) mirror_publication: crate::storage::PreparedMirrorPublication,
    pub(super) prepared_history: crate::mvcc::PreparedRelationalPublicationAccelerators,
    pub(super) validated_lineage_events: crate::runtime::ValidatedLineageEventBatch,
    pub(super) lineage_nodes: Vec<crate::lineage::data::LineageNode>,
    pub(super) artifacts: crate::storage::overlay::PublicationArtifacts,
    pub(super) changed_records: Vec<crate::transactions::data::RecordRef>,
    pub(super) version_id: VersionId,
    pub(super) previous_branch_head_version: Option<VersionId>,
    pub(super) commit_id: CommitId,
    pub(super) commit_reference: RelationalCommitReceipt,
    pub(super) branch_id: BranchId,
    pub(super) merge_base_commits: Vec<CommitId>,
    pub(super) merge_parent_branches: Vec<BranchId>,
    pub(super) deferred_diagnostic_artifacts:
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
}

impl PreparedPublicationPhase {
    pub(super) fn descriptive_touches(
        &self,
    ) -> &crate::history::data::RelationalDescriptiveTouchGraph {
        self.prepared_history.descriptive_touches()
    }

    pub(super) fn changed_record_count(&self) -> usize {
        self.changed_records.len()
    }

    pub(super) fn reservation_count(&self) -> usize {
        self.mutation_identities.records.reservation_count()
    }

    pub(super) fn prepared_root(&self) -> &Arc<crate::branch::RelationalBranchRoot> {
        self.prepared_history.root()
    }

    #[cfg(test)]
    pub(super) fn materialization_counts(&self) -> (u64, u64) {
        self.prepared_history.materialization_counts()
    }

    pub(crate) fn split(
        self,
    ) -> (
        crate::mvcc::PreparedCanonicalBranchMovement,
        PreparedPublicationCompletion,
    ) {
        let (next_cell, root, _envelope, prepared_history) =
            self.prepared_history.into_canonical_parts();
        let movement = crate::mvcc::PreparedCanonicalBranchMovement::new(
            self.mutation_identities,
            next_cell,
            root,
            self.canonical_publication_route,
        );
        let completion = PreparedPublicationCompletion {
            mirror_publication: self.mirror_publication,
            prepared_history,
            validated_lineage_events: self.validated_lineage_events,
            lineage_nodes: self.lineage_nodes,
            artifacts: self.artifacts,
            changed_records: self.changed_records,
            version_id: self.version_id,
            previous_branch_head_version: self.previous_branch_head_version,
            commit_id: self.commit_id,
            commit_reference: self.commit_reference,
            branch_id: self.branch_id,
            merge_base_commits: self.merge_base_commits,
            merge_parent_branches: self.merge_parent_branches,
            deferred_diagnostic_artifacts: self.deferred_diagnostic_artifacts,
        };
        (movement, completion)
    }
}

pub(super) struct PreparePublicationPhaseInput<'a> {
    pub(super) recovered_codec_publication_custody:
        Option<Arc<crate::durability::RecoveredCodecPublicationCustody>>,
    pub(super) required_output_publication: Option<crate::mvcc::RequiredOutputPublicationAdmission>,
    pub(super) required_output_materialization:
        Option<Arc<crate::history::data::CanonicalRequiredOutputMaterialization>>,
    pub(super) created_entities: &'a crate::transactions::data::CommitCreatedEntityBindings,
    pub(super) created_relations: &'a crate::transactions::data::CommitCreatedRelationBindings,
    pub(super) commit_log: &'a mut CommitLog,
    pub(super) working_state: crate::storage::overlay::WorkingState,
    pub(super) mutation_identities:
        crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
    pub(super) selected_branch_state: crate::branch::SelectedRelationalBranchState,
    pub(super) publication: PublicationPreparation,
    pub(super) version_id: VersionId,
    pub(super) previous_branch_head_version: Option<VersionId>,
    pub(super) commit_id: CommitId,
    pub(super) commit_reference: RelationalCommitReceipt,
    pub(super) branch_basis: crate::branch::AdmittedRelationalBranchBasis,
    pub(super) branch_id: BranchId,
    pub(super) merge_base_commits: Vec<CommitId>,
    pub(super) merge_parent_branches: Vec<BranchId>,
}

pub(super) fn prepare_publication_phase(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    input: PreparePublicationPhaseInput<'_>,
) -> Result<PreparedPublicationPhase, TransactionCommitError> {
    let PreparePublicationPhaseInput {
        recovered_codec_publication_custody,
        required_output_publication,
        required_output_materialization,
        created_entities,
        created_relations,
        commit_log,
        working_state,
        mutation_identities,
        selected_branch_state,
        publication,
        version_id,
        previous_branch_head_version,
        commit_id,
        commit_reference,
        branch_basis,
        branch_id,
        merge_base_commits,
        merge_parent_branches,
    } = input;
    let (
        artifacts,
        changed_records,
        mut canonical_commit_envelope,
        _adjacency_deltas,
        lineage_nodes,
        deferred_diagnostic_artifacts,
        target_schema_registry,
    ) = publication.into_finalize().into_parts();
    if required_output_materialization.is_some()
        && (recovered_codec_publication_custody.is_some() || required_output_publication.is_some())
    {
        return Err(crate::mvcc::origin_denial(
            "materialization maintenance cannot also publish producer or codec retirement meaning",
        ));
    }
    canonical_commit_envelope.required_output_materialization = required_output_materialization;
    if recovered_codec_publication_custody.is_some() && required_output_publication.is_some() {
        return Err(crate::mvcc::origin_denial(
            "codec retirement cannot also mint a producer completion",
        ));
    }
    let output_positions = if let Some(custody) = recovered_codec_publication_custody {
        Some(custody.install_retirements(
            execution,
            &mut canonical_commit_envelope,
            &branch_basis,
        )?)
    } else if let Some(admission) = required_output_publication {
        Some(canonical_commit_envelope.install_required_output_origin(
            execution,
            runtime,
            &branch_basis,
            created_entities,
            created_relations,
            &working_state,
            admission,
        )?)
    } else {
        None
    };
    let canonical_commit_envelope = Arc::new(canonical_commit_envelope);
    let clone_mode = working_state.clone_mode();
    let committed_partitions = working_state.into_partition_commits().1;
    runtime
        .history
        .validate_branch_root_capture(committed_partitions.len())
        .map_err(|denial| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                format!("branch-root capture preflight denied: {denial:?}"),
            ))
            .with_commit_log(commit_log.clone())
        })?;
    let published_partition_delta =
        crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
            &committed_partitions,
        );
    let publication_schema_registry = target_schema_registry
        .or_else(|| {
            selected_branch_state
                .root()
                .map(|root| root.schema_authority().registry_arc())
        })
        .unwrap_or_else(|| std::sync::Arc::new(runtime.config.schema.registry.clone()));
    let prepared_history = runtime
        .mvcc_publication_authority()
        .prepare_commit(
            commit_id,
            commit_reference.clone(),
            &branch_basis,
            &selected_branch_state,
            published_partition_delta,
            Arc::clone(&canonical_commit_envelope),
            publication_schema_registry.as_ref(),
            output_positions,
        )
        .map_err(|denial| {
            preparation_denial::transaction_error(denial, &branch_basis, commit_log.clone())
        })?;
    runtime
        .validate_reserved_lineage_events(canonical_commit_envelope.lineage_events())
        .map_err(|detail| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                detail,
            ))
            .with_commit_log(commit_log.clone())
        })?;
    let validated_lineage_events = crate::runtime::ValidatedLineageEventBatch::from_reserved(
        canonical_commit_envelope.lineage_events().to_vec(),
    );
    let canonical_publication_route = runtime
        .history
        .reserve_canonical_publication_route(
            Arc::clone(&canonical_commit_envelope),
            Arc::clone(prepared_history.root()),
        )
        .map_err(|detail| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                detail,
            ))
            .with_commit_log(commit_log.clone())
        })?;

    let mirror_publication = runtime
        .mirror_residency
        .prepare_publication(
            execution,
            &runtime.services.symbols,
            &branch_id,
            prepared_history.root(),
            clone_mode,
            committed_partitions,
        )
        .map_err(|failure| {
            crate::authority::commit::preparation::storage_read_denial(&branch_basis, failure)
        })?;
    Ok(PreparedPublicationPhase {
        mutation_identities,
        mirror_publication,
        prepared_history,
        validated_lineage_events,
        lineage_nodes,
        artifacts,
        changed_records,
        version_id,
        previous_branch_head_version,
        commit_id,
        commit_reference,
        branch_id,
        merge_base_commits,
        merge_parent_branches,
        deferred_diagnostic_artifacts,
        canonical_publication_route,
    })
}
