//! Original query meaning and parameters co-retained with one performed occurrence.
use std::sync::Arc;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::{
    InstalledInverseBinding, WorthQueryInstalledApplicationQueryIdentity,
};

/// Historical descriptive meaning. Fresh query execution must authorize its use.
pub(in crate::domain_computation) struct OriginalCorrectionDisclosure {
    association: Arc<InstalledInverseBinding>,
    query: WorthQueryInstalledApplicationQueryIdentity,
    parameters: Arc<WorthQueryAdmittedApplicationQueryParameters>,
}

impl OriginalCorrectionDisclosure {
    pub(in crate::domain_computation) fn from_original(
        association: Arc<InstalledInverseBinding>,
        query: WorthQueryInstalledApplicationQueryIdentity,
        parameters: WorthQueryAdmittedApplicationQueryParameters,
    ) -> Self {
        Self {
            association,
            query,
            parameters: Arc::new(parameters),
        }
    }
    pub(in crate::domain_computation) fn association(&self) -> &InstalledInverseBinding {
        &self.association
    }
    pub(in crate::domain_computation) fn query(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.query
    }
    pub(in crate::domain_computation) fn parameters(
        &self,
    ) -> &WorthQueryAdmittedApplicationQueryParameters {
        &self.parameters
    }
}
