//! Original-request canonical reads for publication preparation, with narrow history reach.
use crate::history::data::{CanonicalCommitEnvelope, CodecRetirementClosureStop as Stop, CommitId};
use crate::identity::data::VersionId;
use crate::publication::ownership::read::roster::CurrentPublicationRoster;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};

pub(crate) struct OutputPreparationInputs<'r> {
    pub(crate) history: &'r RelationalPreparationHistory,
    pub(crate) current_ownership: &'r Option<Arc<CurrentPublicationRoster>>,
    pub(crate) runtime_instance_id: u64,
    pub(crate) custom_invariant_generation: u64,
}

pub(super) struct OutputPreparationRequest<'r, 'q, 'l> {
    pub(super) current_ownership: &'r Option<Arc<CurrentPublicationRoster>>,
    pub(super) runtime_instance_id: u64,
    pub(super) custom_invariant_generation: u64,
    pub(super) history: PreparationHistoryRequest<'r, 'q, 'l>,
}
impl<'r, 'q, 'l> OutputPreparationRequest<'r, 'q, 'l> {
    pub(super) fn new(
        inputs: OutputPreparationInputs<'r>,
        execution: &ExecutionRequest<'q, 'l>,
    ) -> Self {
        Self {
            current_ownership: inputs.current_ownership,
            runtime_instance_id: inputs.runtime_instance_id,
            custom_invariant_generation: inputs.custom_invariant_generation,
            history: PreparationHistoryRequest {
                history: inputs.history,
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
