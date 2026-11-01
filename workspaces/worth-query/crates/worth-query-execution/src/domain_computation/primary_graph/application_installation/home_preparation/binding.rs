//! Home descriptors are admitted by the actual installed schema and closed roster.
use super::*;
use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId, HomeProgramBinding};
use worth_proof::AdmittedApplicationHomeBinding;
use worth_query_installation::facade::{
    WorthQueryHomeBindingAdmission, WorthQueryHomeBindingDenial,
};

impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeApplication<Schema, Program> {
    pub fn admitted_home_binding(
        &self,
    ) -> Result<
        AdmittedApplicationHomeBinding<WorthQueryHomeBindingAdmission>,
        WorthQueryHomeBindingDenial,
    > {
        self.state.admitted_home_binding()
    }
}

impl<Schema: ApplicationSchemaComposition, Program> HomeApplicationState<Schema, Program> {
    pub(super) fn admitted_home_binding(
        &self,
    ) -> Result<
        AdmittedApplicationHomeBinding<WorthQueryHomeBindingAdmission>,
        WorthQueryHomeBindingDenial,
    > {
        let installed = &self.installed;
        let binding = installed.binding_identity();
        let programs = self
            .support
            .roster
            .entries()
            .iter()
            .map(|entry| {
                HomeProgramBinding::new(
                    entry.identity().as_str().to_owned(),
                    CanonicalDigestId::new(*entry.revision().as_bytes()),
                )
            })
            .collect::<Vec<_>>();
        let selected = programs
            .iter()
            .find(|entry| {
                entry.revision()
                    == CanonicalDigestId::new(*self.support.initial_revision.as_bytes())
            })
            .cloned()
            .ok_or(WorthQueryHomeBindingDenial::SelectedProgramNotRostered)?;
        let description = ApplicationHomeBinding::new(
            *binding.package_identity(),
            *binding.schema_identity(),
            selected,
            programs,
        )
        .map_err(|_| WorthQueryHomeBindingDenial::DescriptionMismatch)?;
        installed.admit_home_binding(
            &self.support.roster,
            &self.support.initial_revision,
            description,
        )
    }
}

impl<Schema: ApplicationSchemaComposition, Program> ReadmittedHomeNativeMeaning<Schema, Program> {
    pub fn home_bridge_definition(
        &self,
    ) -> Result<
        worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        super::super::super::managed_bridge::home_definition::prepare_home_bridge_definition(
            &self.state.installed,
            &self.schema.layout,
        )
    }
}

impl<Schema: ApplicationSchemaComposition, Program> ReadmittedHomeNativeMeaning<Schema, Program> {
    pub fn home_world_resources(
        &self,
    ) -> (
        worth_runtime_world::facade::RuntimeWorldBudgets,
        worth_runtime_world::facade::RuntimeWorldClock,
        worth_foundational::ExecutionRequestPolicy,
        Option<std::sync::Arc<worth_execution::ExecutionAuthority>>,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) {
        let (budgets, clock, _, execution) = self.state.limits.world.clone().into_parts();
        (
            budgets,
            worth_runtime_world::facade::RuntimeWorldClock::from_source(clock),
            execution.policy,
            execution.authority,
            self.state.limits.conditionals,
        )
    }
}
