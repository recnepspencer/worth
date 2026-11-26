use std::collections::BTreeSet;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyFieldsPort as _,
};
use worth_relational_contracts::internal_port as contracts;

use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::history::data::history_policy::HistoryDriftClass;
use contracts::schema::data::RelationalSchemaRegistry;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::LineageSubsystem;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

use super::super::history_parity::validate_expected_recovery_parent_shape;
use super::super::recovered_counter_capacity::prepare_recovery_lineage_sequence;
use super::super::root_inventory::RecoveredRootInventory;

pub struct ReplayPreparationInputs<'a> {
    pub history: &'a HistorySubsystem,
    pub lineage: &'a LineageSubsystem,
    pub registry: &'a RelationalSchemaRegistry,
    pub symbols: &'a RuntimeSymbolTable,
}

pub fn prepare_replay_context(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: ReplayPreparationInputs<'_>,
    envelope: &CanonicalCommitEnvelope,
    available_commit_ids: &BTreeSet<contracts::history::data::coordinates::CommitId>,
    recovered_roots: &RecoveredRootInventory,
    record_parent_validation: impl FnOnce(usize, usize),
) -> Result<bool, RelationalRecoveryStop> {
    let had_checkpoint_artifact = inputs
        .history
        .try_has_commit(envelope.commit.commit_id)
        .map_err(|cause| commit_index_unavailable(cause))?;
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
        worth_foundational::FoundationalBranchTarget::Basis(target) => recovered_roots.resolve(
            contracts::history::data::coordinates::CommitId(target.selected_commit_id()),
        ),
    };
    let recovered_provenance_root =
        checkpoint
            .fork_provenance
            .as_ref()
            .and_then(|provenance| match provenance.target() {
                worth_foundational::FoundationalBranchTarget::Empty => None,
                worth_foundational::FoundationalBranchTarget::Basis(target) => recovered_roots
                    .resolve(contracts::history::data::coordinates::CommitId(
                        target.selected_commit_id(),
                    )),
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
    available_commit_ids: &BTreeSet<contracts::history::data::coordinates::CommitId>,
    record_parent_validation: impl FnOnce(usize, usize),
) -> Result<(), RelationalRecoveryStop> {
    let authoritative_parent_list = envelope.commit.ordered_parents();
    record_parent_validation(1, authoritative_parent_list.len());
    for parent in authoritative_parent_list.as_slice() {
        let in_history = history
            .try_has_commit(*parent)
            .map_err(commit_index_unavailable)?;
        if !available_commit_ids.contains(parent) && !in_history {
            return Err(DurabilityError::new(
                RecoveryFailureClass::MissingAuthoritativeParentClosure,
                format!(
                    "missing authoritative ordered parent closure for commit {}",
                    envelope.commit.commit_id.0
                ),
            )
            .with_history_drift_class(HistoryDriftClass::CanonicalHistoryDrift)
            .into());
        }
        if !history.has_recorded_commit_envelope(*parent) && !in_history {
            return Err(DurabilityError::new(
                RecoveryFailureClass::MissingAuthoritativeParentClosure,
                format!(
                    "authoritative parent commit not recoverable before child {}",
                    envelope.commit.commit_id.0
                ),
            )
            .with_history_drift_class(HistoryDriftClass::CanonicalHistoryDrift)
            .into());
        }
    }
    Ok(())
}

fn commit_index_unavailable(
    cause: contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
) -> RelationalRecoveryStop {
    crate::durability::data::commit_index_access::stop(0, cause)
}
