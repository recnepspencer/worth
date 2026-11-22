//! Crate-boundary port for `WorthQueryProductRuntime` home attachment consumed
//! by `worth-query-execution`. Never exported on `worth-query-execution::facade`
//! or Host.

use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

use super::{
    HomeMaintenanceAdapter, WorthQueryHomeMaintenance, WorthQueryProductRelationalInstallation,
    WorthQueryProductRuntime, WorthQueryProductRuntimeInstallationDenial,
    WorthQueryProductWorldResources,
};

pub trait QueryProductRuntimeInternalPort: Sized {
    #[allow(clippy::too_many_arguments)]
    fn attach_home_world_ports(
        relational: WorthQueryProductRelationalInstallation,
        bridge: &worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly,
        resources: WorthQueryProductWorldResources,
        publication: worth_runtime_world::facade::RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: worth_runtime_world::facade::RuntimeWorldBranchPort,
        observation: worth_runtime_world::facade::RuntimeWorldObservationPort,
        inspection: worth_runtime_world::facade::RuntimeWorldInspectionPort,
        recovery: worth_runtime_world::facade::RuntimeWorldRecoveryPort,
        root: worth_runtime_world::facade::ProductBranchObservation,
        maintenance: std::sync::Arc<dyn WorthQueryHomeMaintenance>,
        branch_program_source_owner: std::sync::Arc<worth_proof::InstalledBranchProgramSourceOwner>,
        branches: &worth_runtime_world::facade::ObservedProductBranches,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial>;
}

impl QueryProductRuntimeInternalPort for WorthQueryProductRuntime {
    fn attach_home_world_ports(
        relational: WorthQueryProductRelationalInstallation,
        bridge: &worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly,
        resources: WorthQueryProductWorldResources,
        publication: worth_runtime_world::facade::RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: worth_runtime_world::facade::RuntimeWorldBranchPort,
        observation: worth_runtime_world::facade::RuntimeWorldObservationPort,
        inspection: worth_runtime_world::facade::RuntimeWorldInspectionPort,
        recovery: worth_runtime_world::facade::RuntimeWorldRecoveryPort,
        root: worth_runtime_world::facade::ProductBranchObservation,
        maintenance: std::sync::Arc<dyn WorthQueryHomeMaintenance>,
        branch_program_source_owner: std::sync::Arc<worth_proof::InstalledBranchProgramSourceOwner>,
        branches: &worth_runtime_world::facade::ObservedProductBranches,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial> {
        <Self as worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort>::attach_home_world_ports(
            relational,
            bridge,
            resources,
            publication,
            branch,
            observation,
            inspection,
            recovery,
            root,
            HomeMaintenanceAdapter::wrap(maintenance),
            branch_program_source_owner,
            branches,
        )
    }
}
