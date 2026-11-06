//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryActiveDirectGraphExecution {
    #[doc(hidden)]
    pub fn fixture_phase_for_test<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        Self::fixture_phase(self, host)
    }
}

impl WorthQueryActiveWorkflowGraphExecution {
    #[doc(hidden)]
    pub fn fixture_phase_for_test<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        Self::fixture_phase(self, host)
    }
}

impl WorthQueryRunningDirectRun {
    #[doc(hidden)]
    pub fn fixture_phase_for_test<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        Self::fixture_phase(self, host)
    }
}

impl WorthQueryRunningWorkflowRun {
    #[doc(hidden)]
    pub fn fixture_phase_for_test<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        Self::fixture_phase(self, host)
    }
}

impl WorthQueryPausedDirectGraphExecution {
    #[doc(hidden)]
    pub fn fixture_phase_for_test<'scope>(
        &self,
        host: &TestHostRequest<'scope>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        Self::fixture_phase(self, host)
    }
}
