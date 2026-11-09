//! Compose authentic selections without rebasing any producer observation.
use super::*;
use crate::execution::request_stage::checkpoint;

impl VisibilityReadContext<'_> {
    /// Oldest first. Each link is authenticated independently through the same
    /// canonical selection reader. Revision comparisons must cross every link;
    /// temporal order alone never proves that an observation remains current.
    pub fn read_historical_selection_chain(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        selections: &[(&RelationalBranchObservation, &RelationalBranchObservation)],
    ) -> Result<RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        crate::execution::check_request(execution).map_err(Denial::Execution)?;
        let count = selections.len();
        if count == 0 {
            return Err(Denial::NotGraphSelection { commit: None });
        }
        let bytes = (count as u64)
            .checked_mul(std::mem::size_of::<HistoricalSelectionStage>() as u64)
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        execution
            .run_native_with_checkpoint(|context| checkpoint(context, count as u64))
            .map_err(Denial::Request)?
            .0
            .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        let mut stages: Vec<HistoricalSelectionStage> = Vec::with_capacity(count);
        for (selected, restored) in selections {
            let mut read = self.read_historical_selection(execution, selected, restored)?;
            let stage = read.stages.pop().expect("one authenticated selection");
            if stages
                .last()
                .is_some_and(|prior| prior.restored.version_id() > stage.selected.version_id())
            {
                return Err(Denial::SelectedBasisMismatch {
                    commit: stage.envelope.commit.commit_id,
                });
            }
            stages.push(stage);
        }
        Ok(RelationalHistoricalSelectionRead {
            stages,
            routes: std::sync::OnceLock::new(),
            _memory: memory,
        })
    }
}
