use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, RuntimeWorldBootstrapIntent, RuntimeWorldBootstrapOutcome,
    RuntimeWorldClock, RuntimeWorldOwner,
};

use super::{
    activation::WorthQueryProductActivationRegistry,
    owner_cleanup::WorthQueryProductBranchOwnerCleanupRegistry, WorthQueryProductRuntime,
    WorthQueryProductRuntimeInstallationDenial,
};

impl WorthQueryProductRuntime {
    pub fn install(
        relational: super::WorthQueryProductRelationalInstallation,
        bridge: &mut BridgeSealedRuntimeAssembly,
        resources: super::WorthQueryProductWorldResources,
        recovered_relational_authority: Option<
            worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
        >,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial> {
        let (budgets, clock) = resources.into_parts();
        let super::WorthQueryProductRelationalInstallation {
            services: relational_services,
            basis: relational_basis,
            source,
        } = relational;
        if !bridge.readmits_authoritative_source_profile(&source.authoritative_source_profile()) {
            return Err(installation_denial(
                "Runtime Bridge and Relational product source do not share exact owner authority"
                    .to_owned(),
            ));
        }
        let relational_lifecycle = relational_services.lifecycle_port();
        let signal_services = bridge.signal_owner_services();
        let signal_basis_port = signal_services.basis_port();
        let signal_lifecycle = signal_services.lifecycle_port();
        let signal_basis = bridge.admitted_signal_basis().clone();
        let correspondence_basis = bridge.admitted_runtime_world_correspondence_basis().clone();
        let definition_publication = bridge
            .take_runtime_world_signal_definition_publication()
            .map_err(|denial| {
                installation_denial(format!("Signal definition service: {denial:?}"))
            })?;
        let cleanup_capacity = budgets
            .live_product_branches()
            .get()
            .checked_add(budgets.retained_product_unpublished_records().get())
            .ok_or_else(|| installation_denial("Owner cleanup capacity overflow".to_owned()))?;
        let owner_cleanup = WorthQueryProductBranchOwnerCleanupRegistry::new(cleanup_capacity);
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
            .with_signal_services(signal_services)
            .with_signal_definition_publication(definition_publication)
            .with_budgets(budgets)
            .with_clock(RuntimeWorldClock::from_source(clock.clone()))
            .build()
            .map_err(|denial| installation_denial(format!("World installation: {denial:?}")))?;
        let branch = ProductBranchCreationIntent::named("primary")
            .map_err(|denial| installation_denial(format!("Product branch name: {denial:?}")))?;
        let bootstrap_intent = match recovered_relational_authority {
            Some(authority) => {
                let recovered_basis = authority.admit_basis(relational_basis).map_err(|_| {
                    installation_denial(
                        "Recovered Relational authority does not own the product basis".to_owned(),
                    )
                })?;
                RuntimeWorldBootstrapIntent::recovered(
                    branch,
                    recovered_basis,
                    signal_basis,
                    correspondence_basis,
                )
            }
            None => RuntimeWorldBootstrapIntent::new(
                branch,
                relational_basis,
                signal_basis,
                correspondence_basis,
            ),
        };
        let outcome = owner
            .lifecycle_port()
            .bootstrap_root(bootstrap_intent)
            .map_err(|denial| {
                installation_denial(format!("World bootstrap service: {denial:?}"))
            })?;
        let RuntimeWorldBootstrapOutcome::Performed(performed) = outcome else {
            return Err(installation_denial(format!(
                "World bootstrap did not publish: {outcome:?}"
            )));
        };
        let observation = performed.product_branch().clone();
        let recovered_root_authority = performed
            .into_recovered_root_authority()
            .map(std::sync::Arc::new);
        activation.commit(&observation);
        Ok(Self::from_parts(
            owner,
            source,
            activations,
            clock,
            relational_lifecycle,
            signal_basis_port,
            signal_lifecycle,
            owner_cleanup,
            observation.branch_identity().clone(),
            observation.lifecycle_incarnation(),
            recovered_root_authority,
        ))
    }
}

fn installation_denial(subject: String) -> WorthQueryProductRuntimeInstallationDenial {
    WorthQueryProductRuntimeInstallationDenial::new(subject)
}
