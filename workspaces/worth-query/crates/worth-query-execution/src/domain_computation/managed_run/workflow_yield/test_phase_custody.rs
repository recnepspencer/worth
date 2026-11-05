//! A fresh fixture host borrows the yielded run's retained source identity.
use super::*;
impl WorthQueryYieldedWorkflowRun {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::request_execution::TestHostRequest<'scope>,
    ) -> crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(&self.relational_basis)
    }
}

#[path = "test_phase_custody_court_doors_for_test.rs"]
mod court_doors_for_test;
