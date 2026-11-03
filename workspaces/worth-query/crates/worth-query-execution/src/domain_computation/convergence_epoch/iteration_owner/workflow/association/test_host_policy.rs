//! Fixture custody is borrowed from the epoch's actual managed run.
use super::*;
impl WorkflowIteratingEpochAssociation {
    pub(in crate::domain_computation::convergence_epoch) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        host.phase_for_workflow(&self.managed_run)
    }
}

impl
    WorkflowIterationAssociation<crate::domain_computation::WorthQueryActiveWorkflowGraphExecution>
{
    pub(in crate::domain_computation::convergence_epoch) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        self.managed.fixture_phase(host)
    }
}

impl
    WorkflowIterationAssociation<crate::domain_computation::WorthQueryPausedWorkflowGraphExecution>
{
    pub(in crate::domain_computation::convergence_epoch) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        self.managed.fixture_phase(host)
    }
}

impl WorkflowIterationAssociation<crate::domain_computation::WorthQueryYieldedWorkflowRun> {
    pub(in crate::domain_computation::convergence_epoch) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        self.managed.fixture_phase(host)
    }
}
