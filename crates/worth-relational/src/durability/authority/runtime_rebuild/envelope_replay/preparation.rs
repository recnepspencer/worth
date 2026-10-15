use std::collections::BTreeSet;

use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::{CanonicalCommitEnvelope, HistoryDriftClass};
use crate::runtime::RelationalRuntime;

use super::super::history_parity::validate_expected_recovery_parent_shape;
use super::super::recovered_counter_capacity::prepare_recovery_lineage_sequence;
use super::super::root_inventory::RecoveredRootInventory;

pub(super) fn prepare_replay_context(
    restored: &mut RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    position: crate::publication::patch::data::PatchStreamPosition,
    available_commit_ids: &BTreeSet<crate::history::data::CommitId>,
    recovered_roots: &mut RecoveredRootInventory,
) -> Result<bool, DurabilityError> {
    let had_checkpoint_artifact = restored
        .history
        .has_recorded_commit_envelope(envelope.commit.commit_id);
    validate_parent_closure(restored, envelope, available_commit_ids)?;
    admit_carried_branch_checkpoint(restored, envelope, recovered_roots)?;
    restored
        .history
        .require_recovered_branch(&envelope.branch_context)
        .map_err(|detail| DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail))?;
    for branch in &envelope.merge_parent_branches {
        restored
            .history
            .require_recovered_branch(branch)
            .map_err(|detail| {
                DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
            })?;
    }
    validate_expected_recovery_parent_shape(restored, envelope)?;
    restored
        .history
        .prepare_recovery_sequence(envelope.commit.commit_id, envelope.commit.version_id);
    restored.history.advance_canonical_stream_floor(
        crate::publication::patch::data::PatchStreamPosition(position.0.saturating_sub(1)),
    );
    prepare_recovery_lineage_sequence(restored, envelope);
    Ok(had_checkpoint_artifact)
}

fn admit_carried_branch_checkpoint(
    restored: &mut RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    recovered_roots: &RecoveredRootInventory,
) -> Result<(), DurabilityError> {
    let Some(checkpoint) = envelope.branch_cell_checkpoint.clone() else {
        if restored.history.has_branch(&envelope.branch_context) {
            return Ok(());
        }
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!(
                "recovery checkpoint omitted branch cell `{}` and its commit envelope carried no exact admission",
                envelope.branch_context.0
            ),
        ));
    };
    let recovered_root = match checkpoint.observation.target() {
        worth_foundational::FoundationalBranchTarget::Empty => None,
        worth_foundational::FoundationalBranchTarget::Basis(target) => {
            recovered_roots.resolve(crate::history::data::CommitId(target.selected_commit_id()))
        }
    };
    let recovered_provenance_root =
        checkpoint
            .fork_provenance
            .as_ref()
            .and_then(|provenance| match provenance.target() {
                worth_foundational::FoundationalBranchTarget::Empty => None,
                worth_foundational::FoundationalBranchTarget::Basis(target) => recovered_roots
                    .resolve(crate::history::data::CommitId(target.selected_commit_id())),
            });
    restored
        .history
        .admit_recovered_branch_cell(
            checkpoint,
            &envelope.branch_context,
            recovered_root,
            recovered_provenance_root,
            &restored.config.schema.registry,
            &restored.services.symbols.interner_snapshot(),
        )
        .map_err(|detail| DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail))
}

fn validate_parent_closure(
    restored: &mut RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    available_commit_ids: &BTreeSet<crate::history::data::CommitId>,
) -> Result<(), DurabilityError> {
    let authoritative_parent_list = envelope.commit.ordered_parents();
    restored
        .performance_access()
        .count_merge_history_durability_validation(1, authoritative_parent_list.len());
    if authoritative_parent_list
        .as_slice()
        .iter()
        .any(|parent| !available_commit_ids.contains(parent))
    {
        return Err(DurabilityError::new(
            RecoveryFailureClass::MissingAuthoritativeParentClosure,
            format!(
                "missing authoritative ordered parent closure for commit {}",
                envelope.commit.commit_id.0
            ),
        )
        .with_history_drift_class(HistoryDriftClass::CanonicalHistoryDrift));
    }
    if authoritative_parent_list
        .as_slice()
        .iter()
        .any(|parent| !restored.history.has_recorded_commit_envelope(*parent))
    {
        return Err(DurabilityError::new(
            RecoveryFailureClass::MissingAuthoritativeParentClosure,
            format!(
                "authoritative parent commit not recoverable before child {}",
                envelope.commit.commit_id.0
            ),
        )
        .with_history_drift_class(HistoryDriftClass::CanonicalHistoryDrift));
    }
    Ok(())
}
