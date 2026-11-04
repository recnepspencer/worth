//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryIteratingDirectConvergenceEpoch {
    #[doc(hidden)]
    pub fn fixture_phase_for_test<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::request_execution::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase<
        'scope,
    > {
        Self::fixture_phase(self, host)
    }
}
