//! Fixture host custody is distinct from an installed runtime's phase.
use super::{
    phase_custody::{ActiveAdvancement, AdvancementSource},
    *,
};

#[derive(Clone, Copy)]
pub(crate) struct TestHostRequest<'scope> {
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

    pub(crate) fn borrowed_request_for_test(&self) -> ExecutionRequest<'_, '_> {
        self.active
            .execution
            .execution_request()
            .expect("the fixture host is admitted")
    }

    pub(crate) fn execution_request_for(
        &self,
        runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<ExecutionRequest<'_, '_>, WorthQueryForeignAdvancementPhase> {
        self.phase_for_product(runtime)
            .execution_request_for(runtime)?;
        Ok(self.borrowed_request_for_test())
    }

    pub(crate) fn bootstrap_for_test(&self) -> WorthQueryBootstrapAdvancementPhase<'_> {
        WorthQueryBootstrapAdvancementPhase::new(self.borrowed_request_for_test())
    }

    pub(crate) fn phase_for_application<Schema: ApplicationSchema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.phase_for_product(&runtime.product_runtime)
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
        runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_installed(
            AdvancementSource::Relational(
                runtime
                    .source
                    .authoritative_source_profile()
                    .runtime_instance_id(),
            ),
            Some(runtime.owner.owner_identity()),
        )
    }

    pub(crate) fn phase_for_direct(
        &self,
        run: &crate::domain_computation::WorthQueryRunningDirectRun,
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_installed(
            AdvancementSource::Relational(run.execution_source_instance_id()),
            run.execution_owner_identity(),
        )
    }

    pub(crate) fn phase_for_workflow(
        &self,
        run: &crate::domain_computation::WorthQueryRunningWorkflowRun,
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_installed(
            AdvancementSource::Relational(run.execution_source_instance_id()),
            run.execution_owner_identity(),
        )
    }

    pub(in crate::domain_computation) fn phase_for_managed(
        &self,
        observation: &crate::domain_computation::managed_run::WorthQueryManagedRelationalObservation,
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.lend_installed(
            AdvancementSource::Relational(observation.identity().runtime_instance_id()),
            observation.owner_identity(),
        )
    }
}
