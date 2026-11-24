//! A fixture host lends one serial request to the production invariant seam.
use super::super::{InvariantEngine, InvariantExecutionRequest, InvariantExecutionResult};
impl<'runtime> InvariantEngine<'runtime> {
    pub(super) fn execute_fixture<'state>(
        &self,
        request: InvariantExecutionRequest<'state>,
    ) -> InvariantExecutionResult
    where
        'runtime: 'state,
    {
        let backing = worth_harness::facade::execution_fixture::serial_request();
        self.execute(
            &worth_execution::ExecutionRequest::serial(&backing),
            request,
        )
        .expect("unconstrained invariant fixture request")
    }
}
