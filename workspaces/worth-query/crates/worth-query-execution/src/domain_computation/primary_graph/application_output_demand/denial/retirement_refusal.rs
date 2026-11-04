//! Retain the original typed retirement refusal and Native record subjects.
use super::WorthQueryOutputDemandDenial;
use worth_relational::facade::publication::PublicationError;
impl WorthQueryOutputDemandDenial {
    /// Original Native cascade refusal, including its affected records.
    pub fn authored_retirement_error(&self) -> Option<&PublicationError> {
        self.evidence.as_ref()?.retirement.as_deref()
    }
    pub(in crate::domain_computation::primary_graph) fn with_retirement_error(
        mut self,
        error: Option<&PublicationError>,
    ) -> Self {
        if let Some(error) = error {
            self.evidence_mut().retirement = Some(std::sync::Arc::new(error.clone()));
        }
        self
    }
}
