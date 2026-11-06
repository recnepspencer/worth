//! Sealed test doors over this module's private members.
use super::*;

impl RequiredOutputReadmission {
    #[doc(hidden)]
    pub fn new_for_test(
        source: Arc<dyn Any + Send + Sync>,
        producer: std::sync::OnceLock<Arc<dyn Any + Send + Sync>>,
        limits: WorthQueryOutputDemandLimits,
        retained_program_basis: Option<
            Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
        >,
    ) -> Self {
        Self {
            source,
            producer,
            limits,
            retained_program_basis,
        }
    }
}
