//! Canonical body reads retain the actual request through reconstructive validation.
use super::CodecRetirementClosureStop as Stop;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;
use crate::schema::data::AspectContractPlanCatalog;
use crate::visibility::materialization::read_records::projection::ProjectionInputs;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};

#[derive(Clone, Copy)]
pub(crate) struct RetirementHistoryInputs<'r> {
    pub(crate) plans: &'r AspectContractPlanCatalog,
    pub(crate) projection: ProjectionInputs<'r>,
    pub(crate) deletion_grammar:
        super::codec_disposition::deletion_grammar::DeletionPublicationGrammar,
}

pub(crate) struct RuntimeHistoryRequest<'r, 'q, 'l> {
    pub(crate) input: RetirementHistoryInputs<'r>,
    execution: ExecutionRequest<'q, 'l>,
}
impl<'r, 'q, 'l> RuntimeHistoryRequest<'r, 'q, 'l> {
    pub(crate) fn execution(&self) -> ExecutionRequest<'q, 'l> {
        self.execution
    }

    pub(crate) fn new(
        input: impl Into<RetirementHistoryInputs<'r>>,
        execution: &ExecutionRequest<'q, 'l>,
    ) -> Self {
        Self {
            input: input.into(),
            execution: *execution,
        }
    }

    pub(crate) fn commit(
        &self,
        id: CommitId,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
        self.input
            .projection
            .history
            .demand_commit_envelope(&self.execution, id)
            .map_err(Stop::Request)
    }

    pub(crate) fn version(
        &self,
        version: VersionId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
        let id = self
            .input
            .projection
            .history
            .commit_for_version_in_context(version, context)
            .map_err(Stop::Request)?;
        match id {
            Some(id) => self.commit(id),
            None => Ok(None),
        }
    }
}
