use std::sync::Arc;

use super::super::artifact_execution::preparation::PublicationPreparation;
use crate::authority::mutation::apply_adjacency_deltas;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use crate::publication::bundle::PublicationStage;
use crate::publication::data::PublicationError;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{CommitLog, TransactionCommitError};

pub(super) struct PreparedPublicationPhase {
    pub(super) durable_bytes: Option<worth_execution::ExecutionImmutableBytes>,
    pub(super) record_allocations: crate::runtime::PendingRecordAllocations,
    pub(super) clone_mode: crate::storage::overlay::PartitionCloneMode,
    pub(super) committed_partitions: std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        (
            crate::storage::overlay::PartitionState,
            crate::storage::overlay::PartitionMutationJournal,
        ),
    >,
    pub(super) prepared_history: crate::mvcc::PreparedRelationalPublication,
    pub(super) validated_lineage_events: crate::runtime::ValidatedLineageEventBatch,
    pub(super) lineage_nodes: Vec<crate::lineage::data::LineageNode>,
    pub(super) artifacts: crate::storage::overlay::PublicationArtifacts,
    pub(super) changed_records: Vec<crate::transactions::data::RecordRef>,
    pub(super) append_authority: crate::durability::authority::DurableAppendAuthority,
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
    pub(super) durable_bytes: Option<worth_execution::ExecutionImmutableBytes>,
    pub(super) clone_mode: crate::storage::overlay::PartitionCloneMode,
    pub(super) committed_partitions: std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        (
            crate::storage::overlay::PartitionState,
            crate::storage::overlay::PartitionMutationJournal,
        ),
    >,
    pub(super) prepared_history: crate::mvcc::PreparedRelationalPublicationAccelerators,
    pub(super) validated_lineage_events: crate::runtime::ValidatedLineageEventBatch,
    pub(super) lineage_nodes: Vec<crate::lineage::data::LineageNode>,
    pub(super) artifacts: crate::storage::overlay::PublicationArtifacts,
    pub(super) changed_records: Vec<crate::transactions::data::RecordRef>,
    pub(super) append_authority: crate::durability::authority::DurableAppendAuthority,
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
    pub(super) fn changed_record_count(&self) -> usize {
        self.changed_records.len()
    }

    pub(super) fn reservation_count(&self) -> usize {
        self.record_allocations.reservation_count()
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
            self.record_allocations,
            next_cell,
            root,
            self.canonical_publication_route,
        );
        let completion = PreparedPublicationCompletion {
            durable_bytes: self.durable_bytes,
            clone_mode: self.clone_mode,
            committed_partitions: self.committed_partitions,
            prepared_history,
            validated_lineage_events: self.validated_lineage_events,
            lineage_nodes: self.lineage_nodes,
            artifacts: self.artifacts,
            changed_records: self.changed_records,
            append_authority: self.append_authority,
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
    pub(super) required_output_publication: Option<crate::mvcc::RequiredOutputPublicationAdmission>,
    pub(super) created_entities: &'a crate::transactions::data::CommitCreatedEntityBindings,
    pub(super) commit_log: &'a mut CommitLog,
    pub(super) working_state: crate::storage::overlay::WorkingState,
    pub(super) record_allocations: crate::runtime::PendingRecordAllocations,
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
        required_output_publication,
        created_entities,
        commit_log,
        working_state,
        record_allocations,
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
        adjacency_deltas,
        lineage_nodes,
        deferred_diagnostic_artifacts,
        target_schema_registry,
    ) = publication.into_finalize().into_parts();
    if let Some(admission) = required_output_publication {
        canonical_commit_envelope.install_required_output_origin(
            execution,
            runtime,
            &branch_basis,
            created_entities,
            &working_state,
            admission,
        )?;
    }
    let durable_bytes = prepare_durable_bytes(execution, runtime, &canonical_commit_envelope)
        .map_err(|error| error.with_commit_log(commit_log.clone()))?;
    let canonical_commit_envelope = Arc::new(canonical_commit_envelope);
    let mut working_state = working_state;
    apply_adjacency_deltas(&mut working_state, &adjacency_deltas, version_id);
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
        )
        .map_err(|detail| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                detail,
            ))
            .with_commit_log(commit_log.clone())
        })?;
    let append_authority = crate::durability::authority::DurableAppendAuthority::from_commit(
        super::CommitDurableAppendAdmission::new(
            runtime.runtime_instance_id(),
            commit_id,
            &branch_id,
        ),
    );
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

    Ok(PreparedPublicationPhase {
        durable_bytes,
        record_allocations,
        clone_mode,
        committed_partitions,
        prepared_history,
        validated_lineage_events,
        lineage_nodes,
        artifacts,
        changed_records,
        append_authority,
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

/// Pre-movement bytes retain their ORIGINAL selected payer through settlement.
fn prepare_durable_bytes(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
) -> Result<Option<worth_execution::ExecutionImmutableBytes>, TransactionCommitError> {
    use crate::durability::data::{
        DurabilityMode, RelationalNativeCheckpointCaptureDenial as Denial,
    };
    if runtime.config.durability.policy.mode != DurabilityMode::PersistedSegmentedLocalFs {
        return Ok(None);
    }
    let publication_error = |detail| {
        TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            detail,
        ))
    };
    let layout = runtime
        .config
        .durability
        .policy
        .store_layout
        .as_ref()
        .ok_or_else(|| {
            publication_error("persisted durability requires its native store layout".to_owned())
        })?;
    crate::durability::log::local_store::preflight_store(execution, layout).map_err(|stop| {
        match stop {
            crate::durability::data::RelationalRecoveryStop::Request(cause) => {
                TransactionCommitError::execution(
                    crate::transactions::data::CommitExecutionDenial {
                        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
                        partition_identity: None,
                    },
                )
            }
            crate::durability::data::RelationalRecoveryStop::Durability(error) => {
                publication_error(error.detail)
            }
            crate::durability::data::RelationalRecoveryStop::Allocation(denial) => {
                crate::execution::allocation_commit_error(TransactionCommitError::conflict(
                    crate::transactions::data::CommitConflict::new(
                        crate::transactions::data::ConflictClass::ExecutionAllocationDenied {
                            denial,
                        },
                    ),
                ))
            }
        }
    })?;
    crate::durability::log::native_file_codec::encode_payload(
        envelope,
        execution.allocation_policy(),
    )
    .map(Some)
    .map_err(|denial| match denial {
        Denial::Allocation(denial) => crate::execution::allocation_commit_error(
            TransactionCommitError::conflict(crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::ExecutionAllocationDenied { denial },
            )),
        ),
        Denial::Durability(error) => publication_error(error.detail),
    })
}
