//! Fixture phases use the retained state's actual installed observation.
use super::*;
use crate::domain_computation::primary_graph::{TestHostRequest, WorthQueryAdvancementPhase};

impl WorthQueryRunningDirectRun {
    pub fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(&self.relational_basis)
    }
}

impl WorthQueryActiveDirectGraphExecution {
    pub fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(&self.running.relational_basis)
    }
}

impl WorthQueryPausedDirectGraphExecution {
    pub fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(&self.active.running.relational_basis)
    }
}

impl WorthQueryRunningWorkflowRun {
    pub fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(self.relational_basis())
    }
}

impl WorthQueryActiveWorkflowGraphExecution {
    pub fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(self.running.relational_basis())
    }
}

impl WorthQueryPausedWorkflowGraphExecution {
    pub fn fixture_phase<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        host.phase_for_managed(self.active.running.relational_basis())
    }
}
