//! Bind selected materialized root images to their canonical head envelopes.
use crate::durability::data::{DurabilityError, DurableCheckpoint};
use crate::runtime::HistorySubsystem;
pub(super) struct ReadmittedRootImages {
    pub(super) roots: std::collections::BTreeMap<
        crate::history::data::CommitId,
        std::sync::Arc<crate::branch::RelationalBranchRoot>,
    >,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(super) fn readmit_selected_images(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    history: &HistorySubsystem,
    checkpoint: &DurableCheckpoint,
    branch_roots: &mut super::branch_root_images::RestoredBranchRootImages,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<ReadmittedRootImages, crate::durability::data::RelationalRecoveryStop> {
    let mut roots = std::collections::BTreeMap::new();
    let mut memory = execution
        .reserve_retained_memory(0)
        .map_err(|cause| crate::durability::data::RelationalRecoveryStop::Request(cause.into()))?;
    for (commit_id, partitions) in std::mem::take(&mut branch_roots.partitions) {
        let original = history.original_root_evidence_for_request(execution, commit_id)?;
        let descriptor = history
            .commit_artifact(commit_id)
            .map(|artifact| artifact.roots().clone())
            .ok_or_else(|| {
                DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    "root image has no actual admitted artifact descriptor",
                )
            })?;
        let schema = branch_roots
            .schema_authorities
            .get(&commit_id)
            .cloned()
            .ok_or_else(|| {
                DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
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
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
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
                crate::branch::RelationalBranchRootCaptureDenial::Content(cause) => {
                    crate::durability::data::RelationalRecoveryStop::from(cause)
                }
                crate::branch::RelationalBranchRootCaptureDenial::StorageUnavailable(cause) => {
                    crate::durability::data::RelationalRecoveryStop::from(cause)
                }
                crate::branch::RelationalBranchRootCaptureDenial::Execution(cause)
                | crate::branch::RelationalBranchRootCaptureDenial::Request(cause) => {
                    crate::durability::data::RelationalRecoveryStop::from(cause)
                }
                other => DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    format!("root readmission: {other:?}"),
                )
                .into(),
            })?;
        let count = roots
            .len()
            .checked_add(1)
            .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
        let bytes = crate::branch::retained_compilation_tree_bytes::<
            crate::history::data::CommitId,
            std::sync::Arc<crate::branch::RelationalBranchRoot>,
        >(count)
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
        crate::durability::artifact_payload_emission::request::account_work(
            execution,
            (u64::from(usize::BITS - count.leading_zeros()) + 1) * 22,
        )?;
        if let Some(memory) = &mut memory {
            memory.resize(bytes).map_err(|cause| {
                crate::durability::data::RelationalRecoveryStop::Request(
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
