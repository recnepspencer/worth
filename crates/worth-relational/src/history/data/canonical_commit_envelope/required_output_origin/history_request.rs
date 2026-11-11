//! Original-request canonical reads for publication preparation, with narrow history reach.
use crate::history::data::{CanonicalCommitEnvelope, CodecRetirementClosureStop as Stop, CommitId};
use crate::identity::data::VersionId;
use crate::runtime::{RelationalPreparationHistory, RelationalPreparationRuntime};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};

pub(super) struct PreparationRuntimeRequest<'r, 'q, 'l> {
    pub(super) runtime: &'r RelationalPreparationRuntime,
    pub(super) history: PreparationHistoryRequest<'r, 'q, 'l>,
}
impl<'r, 'q, 'l> PreparationRuntimeRequest<'r, 'q, 'l> {
    pub(super) fn new(
        runtime: &'r RelationalPreparationRuntime,
        execution: &ExecutionRequest<'q, 'l>,
    ) -> Self {
        Self {
            runtime,
            history: PreparationHistoryRequest {
                history: &runtime.history,
                execution: *execution,
            },
        }
    }
}

pub(super) struct PreparationHistoryRequest<'r, 'q, 'l> {
    history: &'r RelationalPreparationHistory,
    execution: ExecutionRequest<'q, 'l>,
}
impl<'r, 'q, 'l> PreparationHistoryRequest<'r, 'q, 'l> {
    pub(super) fn execution(&self) -> ExecutionRequest<'q, 'l> {
        self.execution
    }

    pub(super) fn canonical_catalog_len(&self) -> usize {
        self.history.canonical_catalog_len()
    }
    pub(super) fn canonical_envelope(
        &self,
        id: CommitId,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
        self.history
            .demand_canonical_envelope(&self.execution, id)
            .map_err(Stop::Request)
    }
    pub(super) fn canonical_envelope_for_version(
        &self,
        version: VersionId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
        let id = self
            .history
            .commit_for_version_in_context(version, context)
            .map_err(Stop::Request)?;
        match id {
            Some(id) => self.canonical_envelope(id),
            None => Ok(None),
        }
    }
}
