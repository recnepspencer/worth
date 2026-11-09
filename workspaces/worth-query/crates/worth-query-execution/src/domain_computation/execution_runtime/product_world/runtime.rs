use worth_relational::facade::branch::RelationalBranchLifecyclePort;
use worth_runtime_bridge::facade::RuntimeBridgeRelationalSource;
use worth_runtime_world::facade::{ProductBranchIdentity, ProductBranchIncarnation};

use std::sync::Arc;

use super::{
    activation::WorthQueryProductActivationRegistry,
    owner_cleanup::WorthQueryProductBranchOwnerCleanupRegistry, WorthQueryProductWorldClock,
};

#[derive(Debug)]
pub(crate) struct WorthQueryProductRootIdentity {
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
    pub(in crate::domain_computation) home_maintenance:
        Option<Arc<dyn super::WorthQueryHomeMaintenance>>,
    pub(super) named_read_probe: Arc<super::read_consult::NamedReadProbe>,
    pub(super) adoption_preparation: Arc<super::adoption_preparation::AdoptionPreparationControl>,
    pub(in crate::domain_computation) directory_support:
        Option<fn(worth_foundational::facade::DurableStateFamily, u8) -> Option<u8>>,
    pub(crate) owner: super::world_ports::WorthQueryProductWorldPorts,
    pub(crate) source: Option<RuntimeBridgeRelationalSource>,
    pub(crate) activations: Arc<WorthQueryProductActivationRegistry>,
    pub(crate) clock: WorthQueryProductWorldClock,
    pub(crate) relational_lifecycle: RelationalBranchLifecyclePort,
    pub(crate) signal_basis: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
    pub(crate) signal_lifecycle: super::signal_lifecycle::SignalLifecycleServices,
    pub(crate) owner_cleanup: WorthQueryProductBranchOwnerCleanupRegistry,
    pub(crate) default_branch: ProductBranchIdentity,
    pub(crate) default_occurrence: ProductBranchIncarnation,
    pub(crate) root_identity: Arc<WorthQueryProductRootIdentity>,
    pub(crate) recovered_root_authority:
        Option<Arc<worth_runtime_world::facade::RecoveredRuntimeWorldRootAuthority>>,
}

impl WorthQueryProductRuntime {
    pub(crate) fn source(&self) -> &RuntimeBridgeRelationalSource {
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

    pub(crate) fn from_parts(
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
            directory_support: None,
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

    pub(crate) fn default_branch(&self) -> &ProductBranchIdentity {
        &self.default_branch
    }

    pub(crate) fn reserve_public_branch_ordinal(
        &self,
    ) -> Result<u64, super::WorthQueryProductBranchCreationDenial> {
        self.owner
            .branch_port()
            .reserve_creation_name_ordinal()
            .map_err(super::WorthQueryProductBranchCreationDenial::World)
    }

    pub(crate) fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Arc::clone(&self.root_identity)
    }
}

mod operation_control;

mod court_doors_for_test;
