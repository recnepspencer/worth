//! Original-request canonical reads for publication preparation, with narrow history reach.
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::history::data::codec_retirement_closure::CodecRetirementClosureStop as Stop;
use branch::publication::ownership::read::roster::CurrentPublicationRoster;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;

pub struct OutputPreparationInputs<'r> {
    pub history: &'r RelationalPreparationHistory,
    pub current_ownership: &'r Option<Arc<CurrentPublicationRoster>>,
    pub runtime_instance_id: u64,
    pub custom_invariant_generation: u64,
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
