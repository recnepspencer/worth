//! Fixture custody follows the owned epoch association.
use super::*;

impl WorthQueryYieldedDirectConvergenceIteration {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::request_execution::TestHostRequest<'scope>,
    ) -> crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'scope> {
        self.association.fixture_phase(host)
    }
}

mod court_doors_for_test;
