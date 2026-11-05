//! A historical verifier admits only source commits on the selected basis's ancestry.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::runtime::{PositionedRelationalSnapshot, RelationalRuntime};
use std::collections::BTreeSet;
impl RelationalHistoricalSelectionRead {
    /// Branch names are not ancestry. Read actual canonical parents, including
    /// across forks; a cold or missing artifact never becomes a positive answer.
    pub fn includes_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        source: &PositionedRelationalSnapshot,
    ) -> Result<bool, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        crate::execution::check_request(execution).map_err(Denial::Execution)?;
        if runtime.runtime_instance_id() != self.restored.runtime_identity
            || source.runtime_instance_id() != self.restored.runtime_identity
        {
            return Err(Denial::ForeignOwner);
        }
        if source.version_id() > self.selected.version_id() {
            return Ok(false);
        }
        let Some(requested) = source.commit_id() else {
            return Ok(false);
        };
        let Some(start) = self
            .selected
            .selected_root()
            .and_then(|root| root.commit_id())
        else {
            return Ok(false);
        };
        let bound = runtime.history.catalog_len();
        let mut visited = BTreeSet::new();
        let initial = execution
            .reserve_retained_memory(512)
            .map_err(Denial::Request)?;
        let mut pending = vec![start];
        let mut memory = vec![initial];
        while let Some(commit) = pending.pop() {
            execution
                .run_native_with_checkpoint(|context| {
                    checkpoint(context, (bound as u64).saturating_add(1))
                })
                .map_err(Denial::Request)?
                .0
                .map_err(|stop| {
                    Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop))
                })?;
            if visited.contains(&commit) {
                continue;
            }
            if visited.len() >= bound {
                return Ok(false);
            }
            memory.push(
                execution
                    .reserve_retained_memory(512)
                    .map_err(Denial::Request)?,
            );
            visited.insert(commit);
            let Some(envelope) = runtime
                .history()
                .demand_commit_envelope(execution, commit)
                .map_err(Denial::Execution)?
            else {
                return Ok(false);
            };
            if commit == requested {
                return Ok(envelope.commit.version_id == source.version_id());
            }
            if envelope.commit.version_id < source.version_id() {
                continue;
            }
            memory.push(
                execution
                    .reserve_retained_memory(
                        (envelope.commit.parents.len() as u64).saturating_mul(32),
                    )
                    .map_err(Denial::Request)?,
            );
            pending.extend(envelope.commit.parents.iter().copied());
        }
        Ok(false)
    }
}
