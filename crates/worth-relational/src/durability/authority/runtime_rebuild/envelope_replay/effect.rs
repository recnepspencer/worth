use std::sync::Arc;

use crate::authority::commit::pipeline::{
    execute_authoritative_commit, AuthoritativeCommitContext,
};
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::history::data::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope};
use crate::runtime::RelationalRuntime;

use super::super::branch_readmission::recovered_branch_basis;
use super::super::history_parity::{
    apply_authoritative_commit_artifacts, validate_recovered_history_parity,
};
use super::super::merge_plan::merge_commit_mutation_plan_from_envelope;
use super::super::recovered_schema_basis::RecoveredSchemaBasis;
use super::owner_bindings::{owner_merge_parent_bases, owner_options_for_branch};

pub(super) fn replay_envelope_effect(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    catalog: crate::durability::authority::RecoveredIndexCatalog,
) -> Result<(), RelationalRecoveryStop> {
    // Replay is the untrusted-input entrance. Admit its existing checkpoint
    // image once, with the registered cold parity check, before native apply.
    restored
        .indexes
        .definition_read_binding()
        .demand_checkpoint_image(execution)
        .map_err(RelationalRecoveryStop::from)?;
    let envelope = positioned.envelope();
    if envelope.authority_kind() == CanonicalCommitAuthorityKind::BranchReferenceMovement
        || is_metadata_only_merge_commit(envelope)
    {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        if governed_merge_authority_was_published(envelope) {
            // Metadata-only replay consumes proof consistency, not a copied
            // mutation plan. Use the same actual published authority predicate.
            if !envelope
                .merge_execution_authority
                .as_ref()
                .is_some_and(|authority| authority.retains_consistent_proof_packet_authority())
            {
                return Err(missing_merge_authority(envelope).into());
            }
        }
        recovered_branch_basis(restored, &envelope.branch_context)?;
        apply_authoritative_commit_artifacts(lane, execution, restored, positioned, false, false)?;
        let branch_basis = recovered_branch_basis(restored, &envelope.branch_context)?;
        let published_partition_delta = restored.storage_authority().affirm_no_partition_changes();
        let prepared_recovery = restored
            .mvcc_publication_authority()
            .prepare_recovered_commit(
                execution,
                envelope.commit.commit_id,
                envelope.commit.clone(),
                &branch_basis,
                published_partition_delta,
                positioned.canonical_arc().clone(),
            )
            .map_err(|denial| match denial {
                crate::mvcc::PublicationPreparationDenial::StorageUnavailable(failure) => {
                    RelationalRecoveryStop::from(failure)
                }
                crate::mvcc::PublicationPreparationDenial::Execution(cause) => {
                    RelationalRecoveryStop::Request(cause)
                }
                crate::mvcc::PublicationPreparationDenial::Invalid(detail) => {
                    DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail).into()
                }
            })?;
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
        replay_ordinary_commit(
            execution,
            restored,
            envelope,
            positioned.position(),
            catalog,
        )?;
    } else {
        replay_merge_commit(
            execution,
            restored,
            envelope,
            positioned.position(),
            catalog,
        )?;
    }
    Ok(())
}

pub(super) fn restore_authoritative_artifacts_when_required(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    restore_authoritative_envelope: bool,
    allow_reconstructed_replacement: bool,
) -> Result<(), RelationalRecoveryStop> {
    let envelope = positioned.envelope();
    if restore_authoritative_envelope {
        validate_recovered_history_parity(restored, envelope)?;
        apply_authoritative_commit_artifacts(
            lane,
            execution,
            restored,
            positioned,
            allow_reconstructed_replacement,
            true,
        )?;
    }
    Ok(())
}

pub(super) fn replay_ordinary_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    position: crate::publication::patch::data::PatchStreamPosition,
    catalog: crate::durability::authority::RecoveredIndexCatalog,
) -> Result<(), RelationalRecoveryStop> {
    crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
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
    txn.content = envelope.content.records().clone();
    txn.stage_recovered_intents(
        execution,
        envelope.commit.commit_id,
        &envelope.merged_plan.merged_intents,
    )
    .map_err(|error| {
        super::commit_stop::replay_commit_stop(
            error,
            "durable transaction copy/staging denied".into(),
        )
    })?;
    let proposal = txn.validate(execution, restored).map_err(|error| {
        let detail = format!("recovered catalog transaction validation: {error:?}");
        super::commit_stop::replay_commit_stop(error, detail)
    })?;
    let context = AuthoritativeCommitContext::from_validated_proposal(proposal)
        .with_recovered_index_catalog(catalog);
    let outcome = execute_authoritative_commit(execution, restored, context).map_err(|error| {
        let detail = format!(
            "failed to replay durable commit {}: {error:?}",
            envelope.commit.commit_id.0
        );
        super::commit_stop::replay_commit_stop(error, detail)
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
    validation.map_err(Into::into)
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    position: crate::publication::patch::data::PatchStreamPosition,
    catalog: crate::durability::authority::RecoveredIndexCatalog,
) -> Result<(), RelationalRecoveryStop> {
    crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
    let merge_plan = require_merge_execution_authority(envelope, execution)?;
    let schema_basis = RecoveredSchemaBasis::admit(restored, envelope)?;
    let mut options = owner_options_for_branch(restored, &envelope.branch_context)?;
    options = options.with_merge_parent_bases(owner_merge_parent_bases(
        restored,
        &envelope.merge_parent_branches,
    )?);
    let context = AuthoritativeCommitContext::from_merge(
        schema_basis
            .apply(options)
            .with_recovered_content(envelope.content.records().clone()),
        merge_plan,
    )
    .map_err(|_| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "failed to reconstruct merge authority context for durable merge commit {}",
                envelope.commit.commit_id.0
            ),
        )
    })?;
    let outcome = execute_authoritative_commit(
        execution,
        restored,
        context.with_recovered_index_catalog(catalog),
    )
    .map_err(|error| {
        let detail = format!(
            "failed to replay durable merge commit {}",
            envelope.commit.commit_id.0
        );
        super::commit_stop::replay_commit_stop(error, detail)
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
    validation.map_err(Into::into)
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
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<crate::mvcc::OwnedMergePlan, RelationalRecoveryStop> {
    merge_commit_mutation_plan_from_envelope(envelope, request)
        .map_err(|error| {
            super::commit_stop::replay_commit_stop(error, "merge plan copy denied".into())
        })?
        .ok_or_else(|| missing_merge_authority(envelope).into())
}

fn missing_merge_authority(envelope: &CanonicalCommitEnvelope) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::ReplayFailure,
        format!(
            "failed to reconstruct merge execution summary for durable merge commit {}",
            envelope.commit.commit_id.0
        ),
    )
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
