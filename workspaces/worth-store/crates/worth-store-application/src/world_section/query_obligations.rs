//! The actual admitted Home lends Query its sole obligation transport.
use super::WorldReadmittedApplicationHome;
use std::sync::Arc;
use worth_query_execution::facade::program_support::{
    QueryObligationsMedium, QueryProgramSupportDenial,
};
impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    /// Closed construction seam for Host's actual Query custody admission.
    pub fn query_obligations_medium(&self) -> Arc<dyn QueryObligationsMedium> {
        self.home.context.query_obligations.clone()
    }
    pub fn admit_query_obligations(
        &self,
        proof: worth_proof::AdmittedHomeQueryObligations,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryProgramSupportDenial> {
        self.home.context.query_obligations.admit(proof, request)
    }
}
