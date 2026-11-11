//! Canonical body reads retain the actual request through reconstructive validation.
use super::CodecRetirementClosureStop as Stop;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;
use crate::runtime::RelationalRuntime;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};

pub(super) struct RuntimeHistoryRequest<'r, 'q, 'l> {
    pub(super) runtime: &'r RelationalRuntime,
    execution: ExecutionRequest<'q, 'l>,
}
impl<'r, 'q, 'l> RuntimeHistoryRequest<'r, 'q, 'l> {
    pub(super) fn execution(&self) -> ExecutionRequest<'q, 'l> {
        self.execution
    }

    pub(super) fn new(
        runtime: &'r RelationalRuntime,
        execution: &ExecutionRequest<'q, 'l>,
    ) -> Self {
        Self {
            runtime,
            execution: *execution,
        }
    }
    pub(super) fn commit(
        &self,
        id: CommitId,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
        self.runtime
            .history_access()
            .demand_commit_envelope(&self.execution, id)
            .map_err(Stop::Request)
    }
    pub(super) fn version(
        &self,
        version: VersionId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
        let id = self
            .runtime
            .history
            .commit_for_version_in_context(version, context)
            .map_err(Stop::Request)?;
        match id {
            Some(id) => self.commit(id),
            None => Ok(None),
        }
    }
}
