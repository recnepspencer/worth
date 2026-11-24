//! Replay declares its metadata-selected ancestor bodies before inspecting them.
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;

pub fn demand_replay_envelopes(
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    ancestor_ids: &BTreeSet<CommitId>,
) -> Result<Vec<Arc<CanonicalCommitEnvelope>>, RelationalRecoveryStop> {
    let mut bodies = Vec::new();
    for id in ancestor_ids {
        if let Some(artifact) = history.commit_artifact(*id) {
            bodies.push(artifact.demand_envelope(execution, 1 << 30)?);
        }
    }
    Ok(bodies)
}
