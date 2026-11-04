//! Fixture phases use the retained state's actual installed observation.
use super::*;
use {
    crate::domain_computation::primary_graph::request_execution::TestHostRequest,
    crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase,
};

impl WorthQueryRunningDirectRun {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(&self.relational_basis)
    }
}

impl WorthQueryActiveDirectGraphExecution {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(&self.running.relational_basis)
    }
}

impl WorthQueryPausedDirectGraphExecution {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(&self.active.running.relational_basis)
    }
}

impl WorthQueryRunningWorkflowRun {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(self.relational_basis())
    }
}

impl WorthQueryActiveWorkflowGraphExecution {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(self.running.relational_basis())
    }
}

impl WorthQueryPausedWorkflowGraphExecution {
    pub(crate) fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(self.active.running.relational_basis())
    }
}

mod court_doors_for_test;
