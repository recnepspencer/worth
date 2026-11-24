use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::data::PublicationSettlementRepairError;
use crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection;
use crate::runtime::state::subsystems::history::history_canonical_routes::{
    publication_requires_settlement, RelationalPublicationHistory,
};

/// Resolve a settled commit identity that no longer has a pending record.
///
/// Return the original receipt from settled performed custody or the actual
/// recovery-admitted checkpoint catalog. This does not promote recovered
/// meaning to a live performed effect or execute derived work again.
pub(crate) fn settled_receipt_without_record(
    history: &RelationalPublicationHistory,
    checkpoint: &RelationalCheckpointManifestSelection,
    commit_id: CommitId,
) -> Result<RelationalCommitReceipt, PublicationSettlementRepairError> {
    if publication_requires_settlement(history, commit_id) {
        return Err(PublicationSettlementRepairError::RecoveryUnavailable { commit_id });
    }
    if let Some(artifact) = history.commit_artifact(commit_id) {
        let cell = artifact.envelope_residency();
        if let Some((entry, source)) = cell.source_placement()
            && let Some(admitted) = checkpoint
                .admitted_commit(&entry)
                .map_err(|_| PublicationSettlementRepairError::RecoveryUnavailable { commit_id })?
        {
            if !checkpoint
                .matches_admitted_source(&admitted, &source)
                || !matches!(
                    history.associated_effect_settlement(&entry, &cell),
                    Some(crate::durability::authority::RelationalAssociatedEffectSettlement::CheckpointReadmitted)
                )
            {
                return Err(PublicationSettlementRepairError::RecoveryUnavailable {
                    commit_id,
                });
            }
            return Ok(admitted.index().commit.clone());
        }
    }
    let positioned = history
        .positioned_canonical_commit(commit_id)
        .ok_or(PublicationSettlementRepairError::RecoveryUnavailable { commit_id })?;
    Ok(positioned.envelope().commit.clone())
}
