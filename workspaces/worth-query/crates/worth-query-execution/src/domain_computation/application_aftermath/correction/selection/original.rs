//! Original query meaning and parameters co-retained with one performed occurrence.
use crate::domain_computation::primary_graph::source_disclosure::OriginalSourceDisclosure;
use std::sync::Arc;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::{
    InstalledCorrectionBinding, WorthQueryInstalledApplicationQueryIdentity,
};

/// Historical descriptive meaning. Fresh query execution must authorize its use.
pub(in crate::domain_computation) struct OriginalCorrectionDisclosure {
    association: Arc<InstalledCorrectionBinding>,
    disclosure: OriginalSourceDisclosure,
}

impl OriginalCorrectionDisclosure {
    pub(in crate::domain_computation) fn from_original(
        association: Arc<InstalledCorrectionBinding>,
        disclosure: OriginalSourceDisclosure,
    ) -> Self {
        Self {
            association,
            disclosure,
        }
    }
    pub(in crate::domain_computation) fn association(&self) -> &InstalledCorrectionBinding {
        &self.association
    }
    pub(in crate::domain_computation) fn query(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryIdentity {
        self.disclosure.query()
    }
    pub(in crate::domain_computation) fn parameters(
        &self,
    ) -> &WorthQueryAdmittedApplicationQueryParameters {
        self.disclosure.parameters()
    }
}
