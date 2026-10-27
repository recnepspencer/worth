use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::CommitId;
use crate::replay::data::RelationalReplayOutcome;
use crate::runtime::RelationalRuntime;

pub struct ReplayAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl<'runtime> ReplayAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub fn canonical_commit_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
    ) -> Result<
        Option<std::sync::Arc<CanonicalCommitEnvelope>>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.runtime
            .history()
            .demand_commit_envelope(execution, commit_id)
    }

    pub fn compare_outcome(&self, outcome: &RelationalReplayOutcome) -> bool {
        outcome.failure.is_none() && outcome.mismatches.is_empty()
    }
}

impl RelationalRuntime {
    pub(crate) fn replay_access(&self) -> ReplayAccess<'_> {
        ReplayAccess::new(self)
    }
}
