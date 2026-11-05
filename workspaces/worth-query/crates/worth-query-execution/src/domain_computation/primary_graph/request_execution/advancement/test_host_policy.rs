//! Fixture host custody is distinct from an installed runtime's phase.
use super::{
    phase_custody::{ActiveAdvancement, AdvancementSource},
    *,
};

#[derive(Clone, Copy)]
pub struct TestHostRequest<'scope> {
    active: &'scope ActiveAdvancement<'scope>,
    installed: Option<(
        AdvancementSource,
        Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    )>,
}

impl<'scope> TestHostRequest<'scope> {
    pub(crate) fn test_host_request(&self) -> Self {
        *self
    }

    pub(super) fn new(active: &'scope ActiveAdvancement<'scope>) -> Self {
        Self {
            active,
            installed: None,
        }
    }

    pub fn borrowed_request_for_test(&self) -> ExecutionRequest<'_, '_> {
        self.active
            .execution
            .execution_request()
            .expect("the fixture host is admitted")
    }

    pub(crate) fn execution_request_for(
        &self,
        runtime: &(impl super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> Result<ExecutionRequest<'_, '_>, WorthQueryForeignAdvancementPhase> {
        self.phase_for_product(runtime)
            .execution_request_for(runtime)?;
        Ok(self.borrowed_request_for_test())
    }

    pub fn bootstrap_for_test(&self) -> WorthQueryBootstrapAdvancementPhase<'_> {
        WorthQueryBootstrapAdvancementPhase::new(self.borrowed_request_for_test())
    }

    pub(crate) fn phase_for_application(
        &self,
        runtime: &(impl super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.phase_for_product(runtime)
    }

    pub(super) fn from_installed(phase: &WorthQueryAdvancementPhase<'scope>) -> Self {
        match phase.source {
            AdvancementSource::Bootstrap => Self::new(phase.active),
            AdvancementSource::Relational(_) | AdvancementSource::ProductOwner(_) => Self {
                active: phase.active,
                installed: Some((phase.source, phase.owner)),
            },
        }
    }

    fn lend_installed(
        &self,
        source: AdvancementSource,
        owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        let (source, owner) = self.installed.unwrap_or((source, owner));
        WorthQueryAdvancementPhase {
            active: self.active,
            source,
            owner,
        }
    }

    pub(crate) fn phase_for_product(
        &self,
        runtime: &(impl super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_source(runtime)
    }

    pub(crate) fn phase_for_direct(
        &self,
        run: &(impl super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_source(run)
    }

    pub(crate) fn phase_for_workflow(
        &self,
        run: &(impl super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_source(run)
    }

    pub(in crate::domain_computation) fn phase_for_managed(
        &self,
        observation: &(impl super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_source(observation)
    }

    fn lend_source(
        &self,
        source: &(impl super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_installed(
            AdvancementSource::Relational(source.advancement_source_instance()),
            source.advancement_owner(),
        )
    }
}

mod court_doors_for_test;
