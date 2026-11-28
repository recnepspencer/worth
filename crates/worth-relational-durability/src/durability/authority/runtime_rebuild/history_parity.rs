use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::history::data::history_policy::HistoryDriftClass;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyFieldsPort as _,
};
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;

pub fn validate_recovered_history_parity(
    recovered_envelope: Option<&CanonicalCommitEnvelope>,
    durable_envelope: &CanonicalCommitEnvelope,
    record_parent_comparisons: impl FnOnce(usize),
) -> Result<(), DurabilityError> {
    let Some(recovered_envelope) = recovered_envelope else {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!(
                "recovered commit envelope missing for durable commit {}",
                durable_envelope.commit.commit_id.0
            ),
        )
        .with_history_drift_class(HistoryDriftClass::DurabilityParityDrift));
    };
    record_parent_comparisons(
        durable_envelope
            .commit
            .ordered_parents()
            .len()
            .max(recovered_envelope.commit.ordered_parents().len()),
    );
    let drifted_axes = canonical_effect_drift(durable_envelope, recovered_envelope);
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
        (*durable.owner_branch_cell_checkpoint()).as_ref(),
        (*replayed.owner_branch_cell_checkpoint()).as_ref(),
    ) {
        drifted.push("branch_cell_checkpoint");
    }
    compare!(authority_kind);
    if durable.owner_index_definition_delta() != replayed.owner_index_definition_delta() {
        drifted.push("index_definition_delta");
    }
    if durable.owner_authored_index_writes() != replayed.owner_authored_index_writes() {
        drifted.push("authored_index_writes");
    }
    compare!(schema_version);
    compare!(schema_authority);
    if durable.merged_plan.merged_intents != replayed.merged_plan.merged_intents {
        drifted.push("merged_plan");
    }
    if durable.record_allocations() != replayed.record_allocations() {
        drifted.push("record_allocations");
    }
    if durable.cancelled_record_allocations() != replayed.cancelled_record_allocations() {
        drifted.push("cancelled_record_allocations");
    }
    if (*durable.owner_revision_symbol_references())
        != (*replayed.owner_revision_symbol_references())
    {
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
    durable: Option<&contracts::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint>,
    replayed: Option<&contracts::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint>,
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
    durable: &contracts::branch::reference::observation::RelationalBranchReferenceObservation,
    replayed: &contracts::branch::reference::observation::RelationalBranchReferenceObservation,
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
    history: &HistorySubsystem,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), DurabilityError> {
    let ordered_parents = envelope.commit.ordered_parents();
    let observed_parents = ordered_parents.as_slice();
    let current_branch_head = history
        .branch_cell(&envelope.branch_context)
        .and_then(|cell| match cell.observation().target() {
            worth_foundational::FoundationalBranchTarget::Basis(target) => Some(
                contracts::history::data::coordinates::CommitId(target.selected_commit_id()),
            ),
            worth_foundational::FoundationalBranchTarget::Empty => None,
        });
    let expected = current_branch_head.as_slice();
    if observed_parents != expected {
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
