//! Bind selected materialized root images to their canonical head envelopes.
use crate::durability::data::{DurabilityError, DurableCheckpoint};
use crate::runtime::HistorySubsystem;

pub(super) fn readmit_selected_images(
    history: &HistorySubsystem,
    checkpoint: &DurableCheckpoint,
    branch_roots: &mut super::branch_root_images::RestoredBranchRootImages,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<
    std::collections::BTreeMap<
        crate::history::data::CommitId,
        std::sync::Arc<crate::branch::RelationalBranchRoot>,
    >,
    DurabilityError,
> {
    let mut roots = std::collections::BTreeMap::new();
    for (commit_id, partitions) in std::mem::take(&mut branch_roots.partitions) {
        let envelope = history.recorded_commit_envelope(commit_id).ok_or_else(|| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                "root has no admitted envelope",
            )
        })?;
        let descriptor =
            checkpoint
                .branch_cells
                .iter()
                .find_map(|cell| match cell.observation.target() {
                    worth_foundational::FoundationalBranchTarget::Basis(target)
                        if target.selected_commit_id() == commit_id.0 =>
                    {
                        Some(target.roots().clone())
                    }
                    _ => None,
                });
        let Some(descriptor) = descriptor else {
            continue;
        };
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
        let root = history
            .readmit_branch_root(partitions, envelope, descriptor, schema, symbols)
            .map_err(|denial| {
                DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    format!("root readmission: {denial:?}"),
                )
            })?;
        roots.insert(commit_id, root);
    }
    Ok(roots)
}
