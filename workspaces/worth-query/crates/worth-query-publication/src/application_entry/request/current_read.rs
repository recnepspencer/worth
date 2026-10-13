//! One ordinary retained read owns its native admission call.
use super::WorthQueryApplicationRequest;
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::WorthQueryProductBranchAdmissionDenial,
};
use worth_query_installation::facade::ApplicationSchema;

/// Why an ordinary current read could not retain its selected product basis.
#[derive(Debug)]
pub enum WorthQueryApplicationReadObservationDenial {
    ExecutionRequest(WorthQueryAdvancementDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
}

impl std::fmt::Display for WorthQueryApplicationReadObservationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutionRequest(cause) => write!(formatter, "read request refused: {cause}"),
            Self::ProductSelection(cause) => write!(formatter, "read selection refused: {cause}"),
        }
    }
}

impl std::error::Error for WorthQueryApplicationReadObservationDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ExecutionRequest(cause) => Some(cause),
            Self::ProductSelection(cause) => Some(cause),
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    /// Retains the selected branch's current basis under this request's native
    /// execution policy, deadline and cancellation. Data reads still resolve
    /// their principal and authorize their own query contracts.
    ///
    /// Call `retain_read(phase)` instead when already inside an admitted host call.
    pub fn retain_current_read(
        &self,
    ) -> Result<
        super::super::WorthQueryApplicationReadObservation,
        WorthQueryApplicationReadObservationDenial,
    > {
        self.application
            .with_application_advancement(self.scope, |phase| self.retain_read(&phase))
            .map_err(WorthQueryApplicationReadObservationDenial::ExecutionRequest)?
            .map_err(WorthQueryApplicationReadObservationDenial::ProductSelection)
    }
}
