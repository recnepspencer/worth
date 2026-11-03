//! A fresh fixture host borrows the yielded run's retained source identity.
use super::*;
impl WorthQueryYieldedDirectRun {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(&self.relational_basis)
    }
}
