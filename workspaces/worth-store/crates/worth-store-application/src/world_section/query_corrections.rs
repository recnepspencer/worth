//! The actual admitted Home lends Query its sole correction journal transport.
use super::WorldReadmittedApplicationHome;
use std::sync::Arc;
use worth_query_execution::facade::correction_material::{
    QueryCorrectionMaterialDenial, QueryCorrectionsMedium,
};
impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    /// Closed construction seam for the actual Home correction owner.
    /// This lends transport only; nonempty correction journals still require
    /// semantic admission before application service.
    pub fn query_corrections_medium(&self) -> Arc<dyn QueryCorrectionsMedium> {
        self.home.context.query_corrections.clone()
    }
    pub fn admit_query_corrections(
        &self,
        proof: worth_proof::AdmittedHomeQueryCorrections,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryCorrectionMaterialDenial> {
        self.home.context.query_corrections.admit(proof, request)
    }
}
