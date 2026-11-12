//! The actual admitted Home lends Host handles over its one dispatch-journal owner.
use super::WorldReadmittedApplicationHome;
use crate::query_dispatch::{
    ApplicationHomeDispatchJournalDenial, ApplicationHomeDispatchJournalOperation,
};
impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    /// Closed construction seam for Host's actual Query dispatch custody admission.
    pub fn query_dispatch_journal_operation(&self) -> ApplicationHomeDispatchJournalOperation {
        ApplicationHomeDispatchJournalOperation::issue(&self.home.context.query_dispatch)
    }
    pub fn admit_query_dispatch(
        &self,
        proof: worth_proof::AdmittedHomeQueryDispatch,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), ApplicationHomeDispatchJournalDenial> {
        self.home.context.query_dispatch.admit(proof, request)
    }
}
