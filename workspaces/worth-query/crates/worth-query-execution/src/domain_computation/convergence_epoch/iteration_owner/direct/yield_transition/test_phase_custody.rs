//! Fixture custody follows the owned epoch association.
use super::*;

impl WorthQueryYieldedDirectConvergenceIteration {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        self.association.fixture_phase(host)
    }
}
