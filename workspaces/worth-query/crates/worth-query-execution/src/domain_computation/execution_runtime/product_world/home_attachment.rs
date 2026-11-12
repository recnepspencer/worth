//! Adopt Home's existing World root and services; this path constructs no owner.
use super::*;
use worth_runtime_world::facade::*;

impl WorthQueryProductRuntime {
    pub(in crate::domain_computation) fn attach_home_world_ports(
        relational: WorthQueryProductRelationalInstallation,
        bridge: &worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly,
        resources: WorthQueryProductWorldResources,
        publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: RuntimeWorldBranchPort,
        observation: RuntimeWorldObservationPort,
        inspection: RuntimeWorldInspectionPort,
        recovery: RuntimeWorldRecoveryPort,
        root: ProductBranchObservation,
        directory_support: fn(worth_foundational::facade::DurableStateFamily, u8) -> Option<u8>,
        maintenance: std::sync::Arc<dyn super::WorthQueryHomeMaintenance>,
        branch_program_source_owner: std::sync::Arc<worth_proof::InstalledBranchProgramSourceOwner>,
        branches: &ObservedProductBranches,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial> {
        let WorthQueryProductRelationalInstallation {
            services,
            basis,
            source,
        } = relational;
        if root.owner_identity().durable().is_none()
            || !root
                .basis()
                .relational_basis()
                .observation()
                .issued_from(&basis)
            || !bridge.readmits_authoritative_source_profile(&source.authoritative_source_profile())
        {
            return Err(home_denial(
                "Home root does not select its exact installed Native/Bridge basis",
            ));
        }
        let ports = world_ports::WorthQueryProductWorldPorts::attached(
            root.owner_identity(),
            publication,
            branch,
            observation,
            inspection,
            recovery,
        )
        .map_err(|_| home_denial("Home World owner is unavailable"))?;
        let (budgets, clock, _, _) = resources.into_parts();
        let cleanup_capacity = match (
            budgets.live_product_branches().get(),
            budgets.retained_product_unpublished_records().get(),
        ) {
            (Some(branches), Some(recovery)) => Some(
                branches
                    .checked_add(recovery)
                    .ok_or_else(|| home_denial("Owner cleanup capacity overflow"))?,
            ),
            _ => None,
        };
        let owner_cleanup =
            owner_cleanup::WorthQueryProductBranchOwnerCleanupRegistry::new(cleanup_capacity);
        let activations = activation::WorthQueryProductActivationRegistry::new(
            budgets.live_product_branches(),
        )
        .map_err(|cause| home_denial(format!("Home activation capacity denied: {cause:?}")))?;
        if !branches.branches().iter().any(|branch| branch == &root) {
            return Err(home_denial(
                "selected Home root is absent from its actual World inventory",
            ));
        }
        for branch in branches.branches() {
            activations
                .reserve()
                .map_err(|cause| home_denial(format!("Home branch activation denied: {cause:?}")))?
                .commit(branch);
        }
        let signal = bridge.signal_owner();
        let mut product = Self::from_parts(
            ports,
            source,
            activations,
            clock,
            services.lifecycle_port(),
            signal.basis_port(),
            super::signal_lifecycle::SignalLifecycleServices::AuthoredHome,
            owner_cleanup,
            root.branch_identity().clone(),
            root.lifecycle_incarnation(),
            None,
        );
        product.branch_program_source_owner = Some(branch_program_source_owner);
        product.directory_support = Some(directory_support);
        product.home_maintenance = Some(maintenance);
        Ok(product)
    }
}

fn home_denial(detail: impl Into<String>) -> WorthQueryProductRuntimeInstallationDenial {
    WorthQueryProductRuntimeInstallationDenial::new(detail.into())
}
