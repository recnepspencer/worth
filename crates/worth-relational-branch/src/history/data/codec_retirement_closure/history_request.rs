//! Canonical body reads retain the actual request through reconstructive validation.
use super::CodecRetirementClosureStop as Stop;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;
use crate::schema::data::AspectContractPlanCatalog;
use crate::visibility::materialization::read_records::projection::ProjectionInputs;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};

#[derive(Clone, Copy)]
pub struct RetirementHistoryInputs<'r> {
    pub plans: &'r AspectContractPlanCatalog,
    pub projection: ProjectionInputs<'r>,
    pub canonical_history: crate::history::envelope_demand::CanonicalHistoryRead<'r>,
    pub deletion_grammar: super::codec_disposition::deletion_grammar::DeletionPublicationGrammar,
}

pub struct RuntimeHistoryRequest<'r, 'q, 'l> {
    pub input: RetirementHistoryInputs<'r>,
    execution: ExecutionRequest<'q, 'l>,
}
impl<'r, 'q, 'l> RuntimeHistoryRequest<'r, 'q, 'l> {
    pub fn execution(&self) -> ExecutionRequest<'q, 'l> {
        self.execution
    }

    pub fn new(
        input: impl Into<RetirementHistoryInputs<'r>>,
        execution: &ExecutionRequest<'q, 'l>,
    ) -> Self {
        Self {
            input: input.into(),
            execution: *execution,
        }
    }

    pub fn commit(&self, id: CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
        self.input
            .canonical_history
            .demand_commit_envelope(&self.execution, id)
            .map_err(Stop::Request)
    }

    pub fn version(
        &self,
        version: VersionId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop> {
        let id = self
            .input
            .canonical_history
            .performed
            .find_version(version, context)
            .map_err(Stop::Request)?;
        match id {
            Some(id) => self.commit(id),
            None => Ok(None),
        }
    }
}
