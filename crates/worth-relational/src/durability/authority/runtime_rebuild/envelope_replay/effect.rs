use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;

use crate::authority::commit::pipeline::execution::execute_authoritative_commit;
use crate::authority::commit::pipeline::AuthoritativeCommitContext;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::history::data::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope};
use crate::runtime::RelationalRuntime;

use super::super::branch_readmission::recovered_branch_basis;
use super::super::history_parity::validate_recovered_history_parity;
use super::super::recovered_schema_basis::RecoveredSchemaBasis;
use super::committed_artifacts::apply_authoritative_commit_artifacts;
use super::owner_bindings::owner_options_for_branch;
use super::path::materialization_mode;

pub(super) fn replay_envelope_effect(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    catalog: crate::durability::authority::RecoveredIndexCatalog,
    recorded_result_schema: Option<crate::durability::authority::runtime_rebuild::recorded_result_schema::RecordedReplayResultSchema>,
) -> Result<(), RelationalRecoveryStop> {
    // Replay is the untrusted-input entrance. Admit its existing checkpoint
    // image once, with the registered cold parity check, before native apply.
    restored
        .indexes
        .definition_read_binding()
        .demand_checkpoint_image(execution)
        .map_err(RelationalRecoveryStop::from)?;
    let envelope = positioned.envelope();
    if envelope.authority_kind() == CanonicalCommitAuthorityKind::BranchReferenceMovement {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        recovered_branch_basis((&*restored).into(), &envelope.branch_context)?;
        apply_authoritative_commit_artifacts(execution, restored, positioned, false, false)?;
        let branch_basis = recovered_branch_basis((&*restored).into(), &envelope.branch_context)?;
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
        restored
            .lineage
            .install_recovered_event_batch(envelope.lineage_events(), envelope.commit.commit_id)
            .map_err(|detail| DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail))?;
        return Ok(());
    }
    super::super::index_catalog::predecessor::reconstruct_predecessor(
        lane,
        execution,
        restored,
        &envelope.branch_context,
        &catalog,
    )?;
    replay_ordinary_commit(
        execution,
        restored,
        envelope,
        positioned.position(),
        catalog,
        recorded_result_schema,
    )
}

pub(super) fn restore_authoritative_artifacts_when_required(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    restore_authoritative_envelope: bool,
    allow_reconstructed_replacement: bool,
) -> Result<(), RelationalRecoveryStop> {
    let envelope = positioned.envelope();
    if restore_authoritative_envelope {
        validate_recovered_history_parity(
            restored
                .history
                .canonical_envelope(envelope.commit.commit_id)
                .as_deref(),
            envelope,
            |count| {
                restored
                    .performance_access()
                    .count_merge_history_parent_comparisons(count);
            },
        )?;
        apply_authoritative_commit_artifacts(
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
    recorded_result_schema: Option<crate::durability::authority::runtime_rebuild::recorded_result_schema::RecordedReplayResultSchema>,
) -> Result<(), RelationalRecoveryStop> {
    crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
    let schema_basis = RecoveredSchemaBasis::admit(
        restored
            .config
            .schema
            .descriptor_canonical_basis_policy
            .current_write_version(),
        envelope,
    )?;
    let options = owner_options_for_branch(restored, &envelope.branch_context)?;
    let options = with_recorded_result_schema(
        configure_recovered_schema(options, &schema_basis),
        recorded_result_schema,
    );
    let options = match materialization_mode(envelope)? {
        Some(mode) => options.with_materialization_mode(mode),
        None => options,
    };
    let mut txn = restored
        .begin_branch_transaction_with_owner_inputs(options)
        .map_err(|error| {
            DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!("failed to admit durable transaction basis: {error:?}"),
            )
        })?;
    txn.content = (*envelope.owner_content()).records().clone();
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
    if let Some(plan) =
        super::naming_recovery::reconstruct_replay_naming_plan(&restored.lineage, envelope)?
    {
        txn.attach_naming_plan(plan);
    }
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

/// A recorded schema-changing commit is judged under its recorded result.
fn with_recorded_result_schema(
    options: crate::mvcc::RelationalTransactionValidationInput,
    recorded: Option<crate::durability::authority::runtime_rebuild::recorded_result_schema::RecordedReplayResultSchema>,
) -> crate::mvcc::RelationalTransactionValidationInput {
    match recorded {
        Some(recorded) => options.with_recorded_result_schema(recorded),
        None => options,
    }
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

fn configure_recovered_schema(
    options: crate::mvcc::RelationalTransactionValidationInput,
    schema_basis: &RecoveredSchemaBasis,
) -> crate::mvcc::RelationalTransactionValidationInput {
    let options = options.with_schema_authority_input(schema_basis.authority_input.clone());
    match schema_basis.proposed_transition.clone() {
        Some(transition) => {
            options.with_schema_transition(transition, schema_basis.reconciliation_policy)
        }
        None => options,
    }
}
