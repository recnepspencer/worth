use std::collections::BTreeSet;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;

use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::history::data::{CanonicalCommitEnvelope, HistoryDriftClass};
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::lineage::LineageSubsystem;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::schema::data::RelationalSchemaRegistry;

use super::super::history_parity::validate_expected_recovery_parent_shape;
use super::super::recovered_counter_capacity::prepare_recovery_lineage_sequence;
use super::super::root_inventory::RecoveredRootInventory;

pub(super) struct ReplayPreparationInputs<'a> {
    pub(super) history: &'a HistorySubsystem,
    pub(super) lineage: &'a LineageSubsystem,
    pub(super) registry: &'a RelationalSchemaRegistry,
    pub(super) symbols: &'a RuntimeSymbolTable,
}

pub(super) fn prepare_replay_context(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: ReplayPreparationInputs<'_>,
    envelope: &CanonicalCommitEnvelope,
    available_commit_ids: &BTreeSet<crate::history::data::CommitId>,
    recovered_roots: &RecoveredRootInventory,
    record_parent_validation: impl FnOnce(usize, usize),
) -> Result<bool, RelationalRecoveryStop> {
    let had_checkpoint_artifact = inputs
        .history
        .has_recorded_commit_envelope(envelope.commit.commit_id);
    validate_parent_closure(
        inputs.history,
        envelope,
        available_commit_ids,
        record_parent_validation,
    )?;
    admit_carried_branch_checkpoint(execution, &inputs, envelope, recovered_roots)?;
    inputs
        .history
        .require_recovered_branch(&envelope.branch_context)
        .map_err(|detail| DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail))?;
    for branch in &envelope.merge_parent_branches {
        inputs
            .history
            .require_recovered_branch(branch)
            .map_err(|detail| {
                DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
            })?;
    }
    validate_expected_recovery_parent_shape(inputs.history, envelope)?;
    inputs
        .history
        .prepare_recovery_sequence(envelope.commit.commit_id, envelope.commit.version_id);
    prepare_recovery_lineage_sequence(inputs.lineage, envelope);
    Ok(had_checkpoint_artifact)
}

fn admit_carried_branch_checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: &ReplayPreparationInputs<'_>,
    envelope: &CanonicalCommitEnvelope,
    recovered_roots: &RecoveredRootInventory,
) -> Result<(), RelationalRecoveryStop> {
    // A fork is the only source of a branch's existence (D8): a commit may
    // carry its branch's prior cell, but never create the branch.
    if !inputs.history.has_branch(&envelope.branch_context) {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!(
                "commit {} is on branch `{}`, which no checkpoint or durable branch transition created",
                envelope.commit.commit_id.0, envelope.branch_context.0
            ),
        ).into());
    }
    let Some(checkpoint) = (*envelope.owner_branch_cell_checkpoint()).clone() else {
        return Ok(());
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
    inputs
        .history
        .admit_recovered_branch_cell(
            execution,
            checkpoint,
            &envelope.branch_context,
            recovered_root,
            recovered_provenance_root,
            inputs.registry,
            |execution, commit, descriptor, root, symbols| {
                crate::runtime::state::subsystems::history::history_catalog_recovery::readmit_replayed_root_descriptor(
                    inputs.history, execution, commit, descriptor, root, symbols,
                )
            },
            &inputs.symbols.interner_snapshot(),
        )
        .map_err(RelationalRecoveryStop::from)
}

fn validate_parent_closure(
    history: &HistorySubsystem,
    envelope: &CanonicalCommitEnvelope,
    available_commit_ids: &BTreeSet<crate::history::data::CommitId>,
    record_parent_validation: impl FnOnce(usize, usize),
) -> Result<(), DurabilityError> {
    let authoritative_parent_list = envelope.commit.ordered_parents();
    record_parent_validation(1, authoritative_parent_list.len());
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
        .any(|parent| !history.has_recorded_commit_envelope(*parent))
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
