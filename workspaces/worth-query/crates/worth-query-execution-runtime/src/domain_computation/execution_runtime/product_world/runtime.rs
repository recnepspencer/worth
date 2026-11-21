use worth_relational::facade::branch::RelationalBranchLifecyclePort;
use worth_runtime_bridge::facade::RuntimeBridgeRelationalSource;
use worth_runtime_world::facade::{ProductBranchIdentity, ProductBranchIncarnation};

use std::sync::Arc;

use super::{
    activation::WorthQueryProductActivationRegistry,
    owner_cleanup::WorthQueryProductBranchOwnerCleanupRegistry, WorthQueryProductWorldClock,
};

#[derive(Debug)]
pub struct WorthQueryProductRootIdentity {
    _private: (),
}

/// One live Query composition of World and its exact source-observation custody.
///
/// Application and installed-domain entry points select through this owner. The
/// default branch is a caller convenience; World alone resolves its current head.
#[derive(Clone)]
pub struct WorthQueryProductRuntime {
    pub(super) branch_program_source_owner:
        Option<Arc<worth_proof::InstalledBranchProgramSourceOwner>>,
    pub home_maintenance: Option<Arc<dyn crate::ports::HomeMaintenanceAfterAdvancement>>,
    pub(super) named_read_probe: Arc<super::read_consult::NamedReadProbe>,
    pub(super) adoption_preparation: Arc<super::adoption_preparation::AdoptionPreparationControl>,
    pub owner: super::world_ports::WorthQueryProductWorldPorts,
    pub source: Option<RuntimeBridgeRelationalSource>,
    pub activations: Arc<WorthQueryProductActivationRegistry>,
    pub clock: WorthQueryProductWorldClock,
    pub relational_lifecycle: RelationalBranchLifecyclePort,
    pub signal_basis: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
    pub signal_lifecycle: super::signal_lifecycle::SignalLifecycleServices,
    pub owner_cleanup: WorthQueryProductBranchOwnerCleanupRegistry,
    pub default_branch: ProductBranchIdentity,
    pub default_occurrence: ProductBranchIncarnation,
    pub root_identity: Arc<WorthQueryProductRootIdentity>,
    pub recovered_root_authority:
        Option<Arc<worth_runtime_world::facade::RecoveredRuntimeWorldRootAuthority>>,
}

impl WorthQueryProductRuntime {
    pub fn source(&self) -> &RuntimeBridgeRelationalSource {
        self.source
            .as_ref()
            .expect("only serving product exposes its source")
    }
    #[doc(hidden)]
    pub fn integration_managed_run_admission<'runtime>(
        &'runtime self,
        query: &'runtime crate::domain_computation::WorthQueryExecutionRuntime,
        bridge: &'runtime worth_runtime_bridge::facade::RuntimeBridge,
    ) -> crate::domain_computation::WorthQueryManagedRunAdmission<'runtime> {
        query.managed_run_admission(bridge, self.source())
    }

    pub fn from_parts(
        owner: super::world_ports::WorthQueryProductWorldPorts,
        source: RuntimeBridgeRelationalSource,
        activations: WorthQueryProductActivationRegistry,
        clock: WorthQueryProductWorldClock,
        relational_lifecycle: RelationalBranchLifecyclePort,
        signal_basis: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
        signal_lifecycle: super::signal_lifecycle::SignalLifecycleServices,
        owner_cleanup: WorthQueryProductBranchOwnerCleanupRegistry,
        default_branch: ProductBranchIdentity,
        default_occurrence: ProductBranchIncarnation,
        recovered_root_authority: Option<
            Arc<worth_runtime_world::facade::RecoveredRuntimeWorldRootAuthority>,
        >,
    ) -> Self {
        #[cfg(not(test))]
        let _ = default_branch;
        Self {
            home_maintenance: None,
            branch_program_source_owner: None,
            owner,
            adoption_preparation: Arc::default(),
            named_read_probe: Arc::default(),
            source: Some(source),
            activations: Arc::new(activations),
            clock,
            relational_lifecycle,
            signal_basis,
            signal_lifecycle,
            owner_cleanup,
            default_branch,
            default_occurrence,
            root_identity: Arc::new(WorthQueryProductRootIdentity { _private: () }),
            recovered_root_authority,
        }
    }

    pub fn owns_branch_selector(&self, branch: crate::basis::WorthQueryProductBranch) -> bool {
        branch.occurrence().owner_identity() == self.owner.owner_identity()
    }

    /// Query token for the installed root product occurrence.
    pub fn root_product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        crate::basis::WorthQueryProductBranch::from_occurrence(self.default_occurrence)
    }

    #[doc(hidden)]
    pub fn integration_owns_product_branch(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> bool {
        branch.occurrence().owner_identity() == self.owner.owner_identity()
    }

    pub fn default_branch(&self) -> &ProductBranchIdentity {
        &self.default_branch
    }

    pub fn reserve_public_branch_ordinal(
        &self,
    ) -> Result<
        u64,
        worth_runtime_world::facade::RuntimeWorldServiceDenial<
            worth_runtime_world::facade::RuntimeWorldBranchAdmissionDenial,
        >,
    > {
        self.owner.branch_port().reserve_creation_name_ordinal()
    }

    pub fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Arc::clone(&self.root_identity)
    }
}

mod operation_control;

mod court_doors_for_test;
