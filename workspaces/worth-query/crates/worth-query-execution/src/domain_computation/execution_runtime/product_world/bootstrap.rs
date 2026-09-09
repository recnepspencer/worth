use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, RuntimeWorldBootstrapIntent, RuntimeWorldBootstrapOutcome,
    RuntimeWorldClock, RuntimeWorldOwner,
};

use super::{
    activation::WorthQueryProductActivationRegistry, WorthQueryProductRuntime,
    WorthQueryProductRuntimeInstallationDenial,
};

impl WorthQueryProductRuntime {
    pub fn install(
        relational: super::WorthQueryProductRelationalInstallation,
        bridge: &mut BridgeSealedRuntimeAssembly,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial> {
        let super::WorthQueryProductRelationalInstallation {
            services: relational_services,
            basis: relational_basis,
            source,
        } = relational;
        let signal_basis = bridge.admitted_signal_basis().clone();
        let correspondence_basis = bridge.admitted_runtime_world_correspondence_basis().clone();
        let definition_publication = bridge
            .take_runtime_world_signal_definition_publication()
            .map_err(|denial| {
                installation_denial(format!("Signal definition service: {denial:?}"))
            })?;
        let clock = super::WorthQueryProductWorldClock::start();
        let budgets = super::installed_budgets();
        let activations = WorthQueryProductActivationRegistry::new(budgets.live_product_branches())
            .map_err(|denial| {
                installation_denial(format!("Product activation capacity: {denial:?}"))
            })?;
        let activation = activations.reserve().map_err(|denial| {
            installation_denial(format!("Root activation admission: {denial:?}"))
        })?;
        let owner = RuntimeWorldOwner::builder()
            .with_bridge_correspondence(bridge.runtime_world_correspondence_port())
            .with_relational_services(relational_services)
            .with_signal_services(bridge.signal_owner_services())
            .with_signal_definition_publication(definition_publication)
            .with_budgets(budgets)
            .with_clock(RuntimeWorldClock::from_source(clock.clone()))
            .build()
            .map_err(|denial| installation_denial(format!("World installation: {denial:?}")))?;
        let branch = ProductBranchCreationIntent::named("primary")
            .map_err(|denial| installation_denial(format!("Product branch name: {denial:?}")))?;
        let outcome = owner
            .lifecycle_port()
            .bootstrap_root(RuntimeWorldBootstrapIntent::new(
                branch,
                relational_basis,
                signal_basis,
                correspondence_basis,
            ))
            .map_err(|denial| {
                installation_denial(format!("World bootstrap service: {denial:?}"))
            })?;
        let RuntimeWorldBootstrapOutcome::Performed(performed) = outcome else {
            return Err(installation_denial(format!(
                "World bootstrap did not publish: {outcome:?}"
            )));
        };
        let observation = performed.product_branch().clone();
        activation.commit(&observation);
        Ok(Self::from_parts(
            owner,
            source,
            activations,
            clock,
            observation.branch_identity().clone(),
        ))
    }
}

fn installation_denial(subject: String) -> WorthQueryProductRuntimeInstallationDenial {
    WorthQueryProductRuntimeInstallationDenial::new(subject)
}
