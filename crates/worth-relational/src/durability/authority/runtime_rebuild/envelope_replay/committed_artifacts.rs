use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::history::data::VersionNode;
use crate::runtime::state::subsystems::history::history_catalog_recovery::{
    record_recovered_commit, source_binding::synchronize_recovered_source_commit,
};
use crate::runtime::RelationalRuntime;

use super::super::super::super::derived_index_artifacts::admit_envelope_derived_index_artifacts;

pub(super) fn apply_authoritative_commit_artifacts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    allow_reconstructed_replacement: bool,
    advance_branch_currentness: bool,
) -> Result<(), RelationalRecoveryStop> {
    runtime
        .unshared_state_mut()
        .ok_or_else(super::super::checkpoint_restore::shared_rebuild_refusal)?;
    let envelope = positioned.envelope();
    let index_artifacts = admit_envelope_derived_index_artifacts(envelope)?;
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
        )
        .into());
    }
    let catalog_matches = runtime
        .history
        .commit_artifact(envelope.commit.commit_id)
        .map(|existing| existing.matches_envelope(envelope))
        .transpose()
        .map_err(|denial| {
            DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                format!("recovered catalog comparison denied: {denial:?}"),
            )
        })?;
    if catalog_matches == Some(false) && !allow_reconstructed_replacement {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!(
                "recovery commit artifact conflicts for commit {}",
                envelope.commit.commit_id.0
            ),
        )
        .into());
    }
    runtime.history.insert_commit_graph_node(
        envelope.commit.commit_id,
        VersionNode {
            commit: envelope.commit.clone(),
        },
    );
    let symbols = runtime.services.symbols.interner_snapshot();
    // Recovery owns an isolated rebuild runtime. Any failure discards it before
    // target publication; copying its entire history here cannot protect a caller.
    (|| {
        let canonical = synchronize_recovered_source_commit(
            &runtime.history,
            execution,
            positioned,
            &symbols,
            |source, residency, roots| {
                runtime
                    .history
                    .canonical_checkpoint_gate()
                    .rebind_recovered_latest(source, residency, roots)
            },
        )?;
        record_recovered_commit(
            &runtime.history,
            execution,
            &canonical,
            allow_reconstructed_replacement,
            advance_branch_currentness,
            &symbols,
        )
    })()
    .map_err(RelationalRecoveryStop::from)?;

    index_artifacts.install(runtime);
    Ok(())
}
