use worth_relational::facade::branch::RelationalBranchLifecyclePort;
use worth_relational::facade::bridge::RuntimeBridgeRelationalSource;
use worth_runtime_world::facade::{
    ProductBranchIdentity, ProductBranchIncarnation, RuntimeWorldOwner,
};

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
    pub(crate) owner: Arc<RuntimeWorldOwner<(), (), (), (), ()>>,
    pub(crate) source: RuntimeBridgeRelationalSource,
    pub(crate) activations: Arc<WorthQueryProductActivationRegistry>,
    pub(crate) clock: WorthQueryProductWorldClock,
    pub(crate) relational_lifecycle: RelationalBranchLifecyclePort,
    pub(crate) signal_basis: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
    pub(crate) signal_lifecycle:
        worth_signal::facade::branch::SignalBranchLifecyclePort<(), (), ()>,
    pub(crate) owner_cleanup: WorthQueryProductBranchOwnerCleanupRegistry,
    #[cfg(test)]
    pub(crate) default_branch: ProductBranchIdentity,
    pub(crate) default_occurrence: ProductBranchIncarnation,
    next_public_branch_ordinal: Arc<std::sync::atomic::AtomicU64>,
    pub(crate) root_identity: Arc<WorthQueryProductRootIdentity>,
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn integration_managed_run_admission<'runtime>(
        &'runtime self,
        query: &'runtime crate::domain_computation::WorthQueryExecutionRuntime,
        bridge: &'runtime worth_runtime_bridge::facade::RuntimeBridge,
    ) -> crate::domain_computation::WorthQueryManagedRunAdmission<'runtime> {
        query.managed_run_admission(bridge, &self.source)
    }

    pub(crate) fn from_parts(
        owner: RuntimeWorldOwner<(), (), (), (), ()>,
        source: RuntimeBridgeRelationalSource,
        activations: WorthQueryProductActivationRegistry,
        clock: WorthQueryProductWorldClock,
        relational_lifecycle: RelationalBranchLifecyclePort,
        signal_basis: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
        signal_lifecycle: worth_signal::facade::branch::SignalBranchLifecyclePort<(), (), ()>,
        owner_cleanup: WorthQueryProductBranchOwnerCleanupRegistry,
        default_branch: ProductBranchIdentity,
        default_occurrence: ProductBranchIncarnation,
    ) -> Self {
        #[cfg(not(test))]
        let _ = default_branch;
        Self {
            owner: Arc::new(owner),
            source,
            activations: Arc::new(activations),
            clock,
            relational_lifecycle,
            signal_basis,
            signal_lifecycle,
            owner_cleanup,
            #[cfg(test)]
            default_branch,
            default_occurrence,
            next_public_branch_ordinal: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            root_identity: Arc::new(WorthQueryProductRootIdentity { _private: () }),
        }
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

    #[cfg(test)]
    pub(crate) fn default_branch(&self) -> &ProductBranchIdentity {
        &self.default_branch
    }

    pub(crate) fn reserve_public_branch_ordinal(&self) -> Option<u64> {
        self.next_public_branch_ordinal
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |current| current.checked_add(1),
            )
            .ok()
    }

    pub(crate) fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Arc::clone(&self.root_identity)
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(crate) fn operation_control(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldOperationControl {
        self.owner.operation_control()
    }
}
