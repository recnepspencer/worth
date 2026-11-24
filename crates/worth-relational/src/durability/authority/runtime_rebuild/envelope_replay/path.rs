//! Classify the replay path from the original recorded envelope.
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::history::data::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope};
use crate::mvcc::transaction::intent::materialization_mode::RelationalMaterializationTransactionMode;

pub(super) fn validate_replay_path(
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), RelationalRecoveryStop> {
    if matches!(
        envelope.authority_kind,
        CanonicalCommitAuthorityKind::OwnedGraphSelection(_)
    ) {
        return Err(DurabilityError::new(
            RecoveryFailureClass::UnsupportedOwnedGraphSelectionReplay,
            "owned graph selection cannot be reconstructed from a canonical tail without its original source custody",
        ).into());
    }
    if matches!(
        envelope.authority_kind,
        CanonicalCommitAuthorityKind::RetainedEntitySelection(_)
    ) {
        return Err(DurabilityError::new(
            RecoveryFailureClass::UnsupportedRetainedEntitySelectionReplay,
            "retained entity selection cannot be reconstructed from a canonical tail without its original source custody",
        ).into());
    }
    Ok(())
}

pub(super) fn is_metadata_only_merge_commit(envelope: &CanonicalCommitEnvelope) -> bool {
    envelope.authority_kind() == CanonicalCommitAuthorityKind::VersionedTransaction
        && !envelope.merge_parent_branches.is_empty()
        && envelope.merged_plan.merged_intents.is_empty()
}

pub(super) fn governed_merge_authority_was_published(envelope: &CanonicalCommitEnvelope) -> bool {
    envelope.merge_execution_authority.is_some()
        || envelope.diagnostics_summary.entries.iter().any(|entry| {
            entry.code == crate::diagnostics::data::DiagnosticCode::MergeExecutionPublished
        })
}

pub(super) fn materialization_mode(
    envelope: &CanonicalCommitEnvelope,
) -> Result<Option<RelationalMaterializationTransactionMode>, DurabilityError> {
    let materialization = envelope
        .merged_plan
        .merged_intents
        .iter()
        .filter_map(|intent| match intent {
            crate::transactions::data::intents::mutation_intent::MutationIntent::Materialization(intent) => Some(intent),
            _ => None,
        })
        .collect::<Vec<_>>();
    if materialization.is_empty() {
        return Ok(None);
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
        (true, false) => RelationalMaterializationTransactionMode::Suspend,
        (false, true) => RelationalMaterializationTransactionMode::Rematerialize,
        _ => {
            return Err(DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                "durable materialization commit has no single reconstruction mode",
            ))
        }
    };
    Ok(Some(mode))
}
