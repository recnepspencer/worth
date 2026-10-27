//! Replay declares its metadata-selected ancestor bodies before inspecting them.
use crate::durability::data::RelationalRecoveryStop;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::runtime::RelationalRuntime;
use std::sync::Arc;

pub(in crate::replay) fn demand_replay_envelopes(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: CommitId,
) -> Result<Vec<Arc<CanonicalCommitEnvelope>>, RelationalRecoveryStop> {
    let ancestry = runtime.history().inspect_commit_ancestry(target);
    let mut bodies = Vec::new();
    for id in ancestry.commit_ids() {
        if let Some(artifact) = runtime.history.commit_artifact(*id) {
            bodies.push(artifact.demand_envelope(execution, 1 << 30)?);
        }
    }
    Ok(bodies)
}
