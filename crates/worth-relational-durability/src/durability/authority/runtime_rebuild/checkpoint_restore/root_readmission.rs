//! Bind selected materialized root images to their canonical head envelopes.
use crate::durability::data::DurableCheckpoint;
use contracts::durability::data::recovery_errors::DurabilityError;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;
pub struct ReadmittedRootImages {
    pub roots: std::collections::BTreeMap<
        contracts::history::data::coordinates::CommitId,
        std::sync::Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
    >,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(super) fn readmit_selected_images(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    history: &HistorySubsystem,
    checkpoint: &DurableCheckpoint,
    branch_roots: &mut super::branch_root_images::RestoredBranchRootImages,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
) -> Result<ReadmittedRootImages, contracts::durability::data::recovery_stop::RelationalRecoveryStop>
{
    let mut roots = std::collections::BTreeMap::new();
    let mut memory = execution.reserve_retained_memory(0).map_err(|cause| {
        contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request(cause.into())
    })?;
    for (commit_id, partitions) in std::mem::take(&mut branch_roots.partitions) {
        let original = history.original_root_evidence_for_request(execution, commit_id)?;
        let descriptor = history
            .commit_artifact(commit_id)
            .map(|artifact| artifact.roots().clone())
            .ok_or_else(|| {
                DurabilityError::new(
                    contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                    "root image has no actual admitted artifact descriptor",
                )
            })?;
        let schema = branch_roots
            .schema_authorities
            .get(&commit_id)
            .cloned()
            .ok_or_else(|| {
                DurabilityError::new(
                    contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                    "root has no schema authority",
                )
            })?;
        let content_records = checkpoint
            .branch_roots
            .iter()
            .find(|image| image.commit_id == commit_id)
            .map(|image| &image.content_records)
            .ok_or_else(|| {
                DurabilityError::new(
                    contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                    "root has no content image",
                )
            })?;
        let root = history
            .readmit_branch_root(
                execution,
                content_records,
                partitions,
                original,
                descriptor,
                schema,
                symbols,
            )
            .map_err(|denial| match denial {
                contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Content(cause) => {
                    contracts::durability::data::recovery_stop::RelationalRecoveryStop::from(cause)
                }
                contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial::StorageUnavailable(cause) => {
                    contracts::durability::data::recovery_stop::RelationalRecoveryStop::from(cause)
                }
                contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Execution(cause)
                | contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Request(cause) => {
                    contracts::durability::data::recovery_stop::RelationalRecoveryStop::from(cause)
                }
                other => DurabilityError::new(
                    contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                    format!("root readmission: {other:?}"),
                )
                .into(),
            })?;
        let count = roots
            .len()
            .checked_add(1)
            .ok_or(contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
        let bytes = contracts::branch::root::schema::standard_tree_quote::retained_tree_bytes::<
            contracts::history::data::coordinates::CommitId,
            std::sync::Arc<
                worth_relational_branch::internal_port::branch::root::RelationalBranchRoot,
            >,
        >(count)
        .map_err(|_| {
            contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow
        })?;
        contracts::durability::data::recovery_stop::account_work(
            execution,
            (u64::from(usize::BITS - count.leading_zeros()) + 1) * 22,
        )?;
        if let Some(memory) = &mut memory {
            memory.resize(bytes).map_err(|cause| {
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request(
                    worth_execution::WorkCeilingDenial::Admission(
                        worth_execution::LeaseDenial::MemoryExhausted(cause),
                    )
                    .into(),
                )
            })?;
        }
        roots.insert(commit_id, root);
    }
    Ok(ReadmittedRootImages {
        roots,
        _memory: memory,
    })
}
