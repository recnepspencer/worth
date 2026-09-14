use std::sync::Arc;

use crate::authority::commit::pipeline::{
    execute_authoritative_commit, AuthoritativeCommitContext,
};
use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope};
use crate::runtime::RelationalRuntime;
use crate::transactions::data::WorkerIntentBatch;

use super::super::branch_readmission::recovered_branch_basis;
use super::super::history_parity::{
    apply_authoritative_commit_artifacts, validate_recovered_history_parity,
};
use super::super::merge_plan::merge_commit_mutation_plan_from_envelope;
use super::super::recovered_schema_basis::RecoveredSchemaBasis;
use super::owner_bindings::{owner_merge_parent_bases, owner_options_for_branch};

pub(super) fn replay_envelope_effect(
    restored: &mut RelationalRuntime,
    positioned: &crate::history::data::PositionedCanonicalCommit,
) -> Result<(), DurabilityError> {
    let envelope = positioned.envelope();
    if envelope.authority_kind() == CanonicalCommitAuthorityKind::BranchReferenceMovement
        || is_metadata_only_merge_commit(envelope)
    {
        if governed_merge_authority_was_published(envelope) {
            require_merge_execution_authority(envelope)?;
        }
        recovered_branch_basis(restored, &envelope.branch_context)?;
        apply_authoritative_commit_artifacts(restored, positioned, false, false)?;
        let branch_basis = recovered_branch_basis(restored, &envelope.branch_context)?;
        let published_partition_delta = restored.storage_authority().affirm_no_partition_changes();
        let prepared_recovery = restored
            .mvcc_publication_authority()
            .prepare_recovered_commit(
                envelope.commit.commit_id,
                envelope.commit.clone(),
                &branch_basis,
                published_partition_delta,
                Arc::clone(positioned.canonical_arc()),
            )
            .map_err(|detail| DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail))?;
        prepared_recovery
            .install_recovered(restored, positioned)
            .map_err(|detail| DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail))?;
        restored
            .history
            .install_recovered_canonical_route(Arc::new(positioned.clone()))
            .map_err(|detail| {
                DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
            })?;
        if envelope.authority_kind() == CanonicalCommitAuthorityKind::BranchReferenceMovement {
            restored
                .lineage
                .install_recovered_event_batch(envelope.lineage_events(), envelope.commit.commit_id)
                .map_err(|detail| {
                    DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail)
                })?;
        }
        return Ok(());
    }
    if envelope.merge_parent_branches.is_empty() {
        replay_ordinary_commit(restored, envelope, positioned.position())?;
    } else {
        replay_merge_commit(restored, envelope, positioned.position())?;
    }
    Ok(())
}

pub(super) fn complete_incomplete_metadata_commit(
    restored: &mut RelationalRuntime,
    readmitted: crate::durability::migration::ReadmittedCanonicalCommit,
) -> Result<crate::history::data::PositionedCanonicalCommit, DurabilityError> {
    let envelope = readmitted.envelope();
    let branch_basis = recovered_branch_basis(restored, &envelope.branch_context)?;
    let published_partition_delta = restored.storage_authority().affirm_no_partition_changes();
    let preflight = restored
        .mvcc_publication_authority()
        .prepare_recovered_commit(
            envelope.commit.commit_id,
            envelope.commit.clone(),
            &branch_basis,
            published_partition_delta,
            Arc::new(envelope.clone()),
        )
        .map_err(|detail| DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail))?;
    let checkpoint = preflight.reconstructed_branch_checkpoint();
    let positioned = readmitted
        .complete_metadata(checkpoint)
        .map_err(|detail| DurabilityError::new(RecoveryFailureClass::CorruptSegment, detail))?;
    replay_envelope_effect(restored, &positioned)?;
    Ok(positioned)
}

pub(super) fn restore_authoritative_artifacts_when_required(
    restored: &mut RelationalRuntime,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    restore_authoritative_envelope: bool,
    allow_reconstructed_replacement: bool,
) -> Result<(), DurabilityError> {
    let envelope = positioned.envelope();
    if restore_authoritative_envelope {
        validate_recovered_history_parity(restored, envelope)?;
        apply_authoritative_commit_artifacts(
            restored,
            positioned,
            allow_reconstructed_replacement,
            true,
        )?;
    }
    Ok(())
}

pub(super) fn replay_ordinary_commit(
    restored: &mut RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    position: crate::publication::patch::data::PatchStreamPosition,
) -> Result<(), DurabilityError> {
    let schema_basis = RecoveredSchemaBasis::admit(restored, envelope)?;
    let mut options = owner_options_for_branch(restored, &envelope.branch_context)?;
    options = options.with_merge_parent_bases(owner_merge_parent_bases(
        restored,
        &envelope.merge_parent_branches,
    )?);
    let options = schema_basis.apply(options);
    let options = apply_materialization_replay_mode(options, envelope)?;
    let mut txn = restored
        .begin_branch_transaction_with_owner_inputs(options)
        .map_err(|error| {
            DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!("failed to admit durable transaction basis: {error:?}"),
            )
        })?;
    txn.push_batch(WorkerIntentBatch {
        name: format!("recovery-commit-{}", envelope.commit.commit_id.0),
        partition_key: None,
        worker_local_only: true,
        intents: envelope.merged_plan.merged_intents.clone().to_vec(),
    })
    .map_err(|denial| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!("durable transaction staging denied: {denial:?}"),
        )
    })?;
    let outcome = restored.commit_branch_transaction(txn).map_err(|error| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "failed to replay durable commit {}: {error:?}",
                envelope.commit.commit_id.0
            ),
        )
    })?;
    let validation = if outcome.patch_position() != position {
        Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            "replayed durable commit stream position drifted",
        ))
    } else {
        schema_basis.validate_replayed(outcome.envelope())
    };
    release_replayed_snapshot(restored, &outcome.snapshot)?;
    validation
}

fn apply_materialization_replay_mode(
    options: crate::mvcc::RelationalTransactionValidationInput,
    envelope: &CanonicalCommitEnvelope,
) -> Result<crate::mvcc::RelationalTransactionValidationInput, DurabilityError> {
    let materialization = envelope
        .merged_plan
        .merged_intents
        .iter()
        .filter_map(|intent| match intent {
            crate::transactions::data::MutationIntent::Materialization(intent) => Some(intent),
            _ => None,
        })
        .collect::<Vec<_>>();
    if materialization.is_empty() {
        return Ok(options);
    }
    if materialization.len() != envelope.merged_plan.merged_intents.len() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            "durable materialization commit mixed owner and ordinary intents",
        ));
    }
    let all_suspend = materialization.iter().all(|intent| intent.is_suspension());
    let all_restore = materialization
        .iter()
        .all(|intent| intent.is_rematerialization());
    let mode = match (all_suspend, all_restore) {
        (true, false) => crate::mvcc::RelationalMaterializationTransactionMode::Suspend,
        (false, true) => crate::mvcc::RelationalMaterializationTransactionMode::Rematerialize,
        _ => {
            return Err(DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                "durable materialization commit has no single reconstruction mode",
            ))
        }
    };
    Ok(options.with_materialization_mode(mode))
}

pub(super) fn replay_merge_commit(
    restored: &mut RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    position: crate::publication::patch::data::PatchStreamPosition,
) -> Result<(), DurabilityError> {
    let merge_plan = require_merge_execution_authority(envelope)?;
    let schema_basis = RecoveredSchemaBasis::admit(restored, envelope)?;
    let mut options = owner_options_for_branch(restored, &envelope.branch_context)?;
    options = options.with_merge_parent_bases(owner_merge_parent_bases(
        restored,
        &envelope.merge_parent_branches,
    )?);
    let context = AuthoritativeCommitContext::from_merge(schema_basis.apply(options), merge_plan)
        .map_err(|_| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "failed to reconstruct merge authority context for durable merge commit {}",
                envelope.commit.commit_id.0
            ),
        )
    })?;
    let outcome = execute_authoritative_commit(restored, context).map_err(|_| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "failed to replay durable merge commit {}",
                envelope.commit.commit_id.0
            ),
        )
    })?;
    let validation = if outcome.patch_position() != position {
        Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            "replayed durable merge stream position drifted",
        ))
    } else {
        schema_basis.validate_replayed(outcome.envelope())
    };
    release_replayed_snapshot(restored, &outcome.snapshot)?;
    validation
}

fn release_replayed_snapshot(
    restored: &mut RelationalRuntime,
    snapshot: &crate::snapshots::data::SnapshotHandle,
) -> Result<(), DurabilityError> {
    restored
        .snapshots()
        .release_snapshot(snapshot)
        .map(|_| ())
        .map_err(|denial| {
            DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!("failed to settle replayed snapshot: {denial:?}"),
            )
        })
}

fn require_merge_execution_authority(
    envelope: &CanonicalCommitEnvelope,
) -> Result<crate::transactions::data::MergeCommitMutationPlan, DurabilityError> {
    merge_commit_mutation_plan_from_envelope(envelope).ok_or_else(|| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "failed to reconstruct merge execution summary for durable merge commit {}",
                envelope.commit.commit_id.0
            ),
        )
    })
}

pub(super) fn is_metadata_only_merge_commit(envelope: &CanonicalCommitEnvelope) -> bool {
    envelope.authority_kind() == CanonicalCommitAuthorityKind::VersionedTransaction
        && !envelope.merge_parent_branches.is_empty()
        && envelope.merged_plan.merged_intents.is_empty()
}

fn governed_merge_authority_was_published(envelope: &CanonicalCommitEnvelope) -> bool {
    envelope.merge_execution_authority.is_some()
        || envelope.diagnostics_summary.entries.iter().any(|entry| {
            entry.code == crate::diagnostics::data::DiagnosticCode::MergeExecutionPublished
        })
}
