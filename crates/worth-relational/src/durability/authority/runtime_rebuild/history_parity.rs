use std::sync::Arc;

use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::{HistoryDriftClass, VersionNode};
use crate::runtime::RelationalRuntime;

use super::super::super::derived_index_artifacts::apply_envelope_derived_index_artifacts;

pub(super) fn apply_authoritative_commit_artifacts(
    runtime: &mut RelationalRuntime,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    allow_reconstructed_replacement: bool,
    advance_branch_currentness: bool,
) -> Result<(), DurabilityError> {
    let envelope = positioned.envelope();
    let history_before = runtime.history.detached_owner_snapshot();
    if runtime
        .history
        .recorded_commit_envelope(envelope.commit.commit_id)
        .is_some_and(|existing| existing.as_ref() != envelope)
        && !allow_reconstructed_replacement
    {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!(
                "recovery commit envelope conflicts for commit {}",
                envelope.commit.commit_id.0
            ),
        ));
    }
    if runtime
        .history
        .commit_artifact(envelope.commit.commit_id)
        .is_some_and(|existing| existing.envelope().as_ref() != envelope)
        && !allow_reconstructed_replacement
    {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!(
                "recovery commit artifact conflicts for commit {}",
                envelope.commit.commit_id.0
            ),
        ));
    }
    runtime.history.insert_commit_graph_node(
        envelope.commit.commit_id,
        VersionNode {
            commit: envelope.commit.clone(),
        },
    );
    let canonical = runtime
        .history
        .recorded_commit_envelope(envelope.commit.commit_id)
        .unwrap_or_else(|| Arc::clone(positioned.canonical_arc()));
    let resident = Arc::new(crate::history::RelationalCommitEnvelopeResidency::resident(
        canonical.clone(),
        canonical
            .encode_authoritative_payload()
            .map(Arc::from)
            .map_err(|error| {
                DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, error.to_string())
            })?,
    ));
    resident.bind_performed(positioned, None);
    runtime.history.with_ledger_mut(|ledger| {
        ledger
            .commit_envelopes
            .insert(envelope.commit.commit_id, resident);
        ledger
            .patch_stream_index
            .insert(positioned.position(), envelope.commit.commit_id);
    });
    let result = runtime
        .history
        .record_recovered_commit(
            canonical.as_ref(),
            allow_reconstructed_replacement,
            advance_branch_currentness,
            &runtime.services.symbols.interner_snapshot(),
        )
        .map_err(|detail| DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail));
    if let Err(error) = result {
        super::unshared_state(runtime)?
            .history
            .restore_detached_recovery_snapshot(history_before);
        return Err(error);
    }

    let artifact = runtime
        .history
        .commit_artifact(envelope.commit.commit_id)
        .ok_or_else(|| {
            DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                "readmitted canonical commit has no catalog entry",
            )
        })?;
    runtime.history.with_ledger_mut(|ledger| {
        ledger
            .commit_envelopes
            .insert(envelope.commit.commit_id, artifact.envelope_residency());
    });
    apply_envelope_derived_index_artifacts(runtime, envelope);
    Ok(())
}

pub(super) fn validate_recovered_history_parity(
    runtime: &mut RelationalRuntime,
    durable_envelope: &CanonicalCommitEnvelope,
) -> Result<(), DurabilityError> {
    let Some(recovered_envelope) = runtime
        .history
        .canonical_envelope(durable_envelope.commit.commit_id)
    else {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "recovered commit envelope missing for durable commit {}",
                durable_envelope.commit.commit_id.0
            ),
        )
        .with_history_drift_class(HistoryDriftClass::DurabilityParityDrift));
    };
    runtime
        .performance_access()
        .count_merge_history_parent_comparisons(
            durable_envelope
                .commit
                .ordered_parents()
                .len()
                .max(recovered_envelope.commit.ordered_parents().len()),
        );
    let drifted_axes = canonical_effect_drift(durable_envelope, &recovered_envelope);
    if !drifted_axes.is_empty() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "recovered durable canonical effect drifted for commit {} on {}",
                durable_envelope.commit.commit_id.0,
                drifted_axes.join(", ")
            ),
        )
        .with_history_drift_class(HistoryDriftClass::DurabilityParityDrift));
    }
    Ok(())
}

fn canonical_effect_drift(
    durable: &CanonicalCommitEnvelope,
    replayed: &CanonicalCommitEnvelope,
) -> Vec<&'static str> {
    let mut drifted = Vec::new();
    macro_rules! compare {
        ($field:ident) => {
            if durable.$field != replayed.$field {
                drifted.push(stringify!($field));
            }
        };
    }
    compare!(commit);
    compare!(branch_context);
    if !branch_cell_checkpoint_truth_matches(
        durable.branch_cell_checkpoint.as_ref(),
        replayed.branch_cell_checkpoint.as_ref(),
    ) {
        drifted.push("branch_cell_checkpoint");
    }
    compare!(authority_kind);
    compare!(merge_execution_authority);
    compare!(merge_parent_branches);
    compare!(merge_base_commits);
    compare!(schema_version);
    compare!(schema_authority);
    if durable.merged_plan.merged_intents != replayed.merged_plan.merged_intents {
        drifted.push("merged_plan");
    }
    if durable.record_allocations() != replayed.record_allocations() {
        drifted.push("record_allocations");
    }
    if durable.revision_symbol_references != replayed.revision_symbol_references {
        drifted.push("revision_symbol_references");
    }
    compare!(patch);
    if durable.published_lineage() != replayed.published_lineage() {
        drifted.push("lineage");
    }
    compare!(schema_transition);
    compare!(schema_continuation_descriptor);
    compare!(schema_reconciliation_descriptor);
    compare!(descriptor_semantics_version);
    drifted
}

fn branch_cell_checkpoint_truth_matches(
    durable: Option<&crate::branch::RelationalBranchCellCheckpoint>,
    replayed: Option<&crate::branch::RelationalBranchCellCheckpoint>,
) -> bool {
    match (durable, replayed) {
        (Some(durable), Some(replayed)) => {
            durable.branch_id == replayed.branch_id
                && branch_observation_truth_matches(&durable.observation, &replayed.observation)
                && durable.truth_version == replayed.truth_version
                && match (
                    durable.fork_provenance.as_ref(),
                    replayed.fork_provenance.as_ref(),
                ) {
                    (Some(durable), Some(replayed)) => {
                        branch_observation_truth_matches(durable, replayed)
                    }
                    (None, None) => true,
                    _ => false,
                }
                && durable.fork_source_branch_id == replayed.fork_source_branch_id
        }
        (None, None) => true,
        _ => false,
    }
}

fn branch_observation_truth_matches(
    durable: &crate::branch::RelationalBranchReferenceObservation,
    replayed: &crate::branch::RelationalBranchReferenceObservation,
) -> bool {
    use worth_foundational::FoundationalBranchTarget;

    if durable.generation() != replayed.generation() {
        return false;
    }
    match (durable.target(), replayed.target()) {
        (FoundationalBranchTarget::Empty, FoundationalBranchTarget::Empty) => true,
        (FoundationalBranchTarget::Basis(durable), FoundationalBranchTarget::Basis(replayed)) => {
            durable.selected_commit_id() == replayed.selected_commit_id()
                && durable.version_id() == replayed.version_id()
                && durable.parent_commit_ids() == replayed.parent_commit_ids()
                && durable.roots() == replayed.roots()
        }
        _ => false,
    }
}

pub(super) fn validate_expected_recovery_parent_shape(
    runtime: &mut RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), DurabilityError> {
    let ordered_parents = envelope.commit.ordered_parents();
    let observed_parents = ordered_parents.as_slice();
    let current_branch_head = runtime
        .history
        .branch_cell(&envelope.branch_context)
        .and_then(|cell| match cell.observation().target() {
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                Some(crate::history::data::CommitId(target.selected_commit_id()))
            }
            worth_foundational::FoundationalBranchTarget::Empty => None,
        });
    // The branch-reference cell is the only currentness authority.  An empty
    // cell is an explicit empty target, not permission to infer a head from
    // the envelope's parent list.
    let admitted_target_head = current_branch_head;

    if envelope.merge_parent_branches.is_empty() {
        let expected = admitted_target_head.into_iter().collect::<Vec<_>>();
        if observed_parents != expected.as_slice() {
            return Err(DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!(
                    "recovered durable history parity drifted for commit {}",
                    envelope.commit.commit_id.0
                ),
            )
            .with_history_drift_class(HistoryDriftClass::DurabilityParityDrift));
        }
        return Ok(());
    }

    let mut expected = Vec::with_capacity(envelope.merge_parent_branches.len() + 1);
    if let Some(target_head) = admitted_target_head {
        expected.push(target_head);
    }
    for branch in &envelope.merge_parent_branches {
        let branch_head = runtime
            .history
            .branch_cell(branch)
            .and_then(|cell| match cell.observation().target() {
                worth_foundational::FoundationalBranchTarget::Basis(target) => {
                    Some(crate::history::data::CommitId(target.selected_commit_id()))
                }
                worth_foundational::FoundationalBranchTarget::Empty => None,
            })
            .ok_or_else(|| {
                DurabilityError::new(
                    RecoveryFailureClass::ReplayFailure,
                    format!(
                        "recovered durable history parity drifted for commit {}",
                        envelope.commit.commit_id.0
                    ),
                )
                .with_history_drift_class(HistoryDriftClass::DurabilityParityDrift)
            })?;
        expected.push(branch_head);
    }
    if observed_parents != expected.as_slice() {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "recovered durable history parity drifted for commit {}",
                envelope.commit.commit_id.0
            ),
        )
        .with_history_drift_class(HistoryDriftClass::DurabilityParityDrift));
    }
    Ok(())
}
