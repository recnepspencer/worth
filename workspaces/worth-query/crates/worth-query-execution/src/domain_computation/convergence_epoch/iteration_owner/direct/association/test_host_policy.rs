//! Fixture custody is borrowed from the epoch's actual managed run.
use super::*;
impl DirectIteratingEpochAssociation {
    pub(in crate::domain_computation::convergence_epoch) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        host.phase_for_direct(&self.managed_run)
    }
}

impl DirectIterationAssociation<crate::domain_computation::WorthQueryActiveDirectGraphExecution> {
    pub(in crate::domain_computation::convergence_epoch) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        self.managed.fixture_phase(host)
    }
}

impl DirectIterationAssociation<crate::domain_computation::WorthQueryPausedDirectGraphExecution> {
    pub(in crate::domain_computation::convergence_epoch) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        self.managed.fixture_phase(host)
    }
}

impl DirectIterationAssociation<crate::domain_computation::WorthQueryYieldedDirectRun> {
    pub(in crate::domain_computation::convergence_epoch) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        self.managed.fixture_phase(host)
    }
}
