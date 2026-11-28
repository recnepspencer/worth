#[cfg(test)]
mod diagnostic_projection;

use crate::publication::bundle::PublicationStage;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::TransactionCommitError;
use crate::validation::engine::InvariantExecutionResult;


impl RelationalRuntime {
    pub(crate) fn certification_invariant_authority(&self) -> CertificationInvariantAuthority<'_> {
        CertificationInvariantAuthority::new(self)
    }

    pub fn certify_current_state(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        self.certification_invariant_authority()
            .enforce_certification_boundary(execution)
    }
}

/// Runtime authority for the explicit certification boundary.
///
/// Ordinary commit preparation uses `PreparationInvariantAuthority` instead.
pub(crate) struct CertificationInvariantAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl<'runtime> CertificationInvariantAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub(crate) fn enforce_certification_boundary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        let result = self.runtime.validation().certification_state(execution)?;
        if let Some(failure) = result.summary().publication_failure() {
            return Err(TransactionCommitError::publication(
                failure
                    .clone()
                    .into_publication_error(PublicationStage::InvariantCheck),
            ));
        }
        Ok(result)
    }
}
