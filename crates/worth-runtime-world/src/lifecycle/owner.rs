use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::signal_services::RuntimeWorldSignalServices;
use worth_relational::facade::branch::RelationalOwnerServicePorts;
use worth_runtime_bridge::facade::RuntimeWorldCorrespondencePort;

use crate::branch::registry::ProductBranchRegistry;
use crate::history::CompositeHistoryCatalog;
use crate::identity::{
    RuntimeWorldIdentityExhaustion, RuntimeWorldIdentityIssuer, RuntimeWorldOwnerIdentity,
};
use crate::recovery::RecoveryCatalog;
use crate::retention::RuntimeWorldRetentionOwner;

use super::clock::RuntimeWorldClock;
use super::close::RuntimeWorldCloseContract;
use super::owner_inputs::{RuntimeWorldInstalledOwnerInputs, RuntimeWorldOwnerInputs};

mod bootstrap;
mod branch_service;
pub(crate) use branch_service::WorldDurableCreationCustody;
mod checkpoint_origin;
mod construction;
mod current_read_publication;
pub use current_read_publication::PreparedWorldRelationalCutover;
pub(crate) use current_read_publication::RelationalCutoverHandoff;
mod execution_service;
mod inspection_service;
mod operation;
mod owned_async_product;
mod publication_service;
mod recovery_service;
#[cfg(test)]
mod rehearsal;

pub(crate) use construction::{
    RuntimeWorldOwnerConstructionCapability, RuntimeWorldOwnerConstructionContract,
};
pub(crate) use operation::{
    ReservedPublicationAttemptCapacity, RuntimeWorldOperationLedger,
    RuntimeWorldOperationLedgerState, RuntimeWorldOperationReservation,
    RuntimeWorldPublicationCapacityLedger,
};
pub use owned_async_product::{
    RuntimeWorldOwnedAsyncManagedRevalidationDenial, RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    RuntimeWorldOwnedAsyncRevalidationDenial,
};

pub(crate) enum RuntimeWorldBootstrapState {
    Unperformed,
    InProgress,
    Performed,
    Pending(Arc<bootstrap::WorldDurableBootstrapCustody>),
}

/// The sole managed Runtime World owner. All identity, budget, component,
/// history, retention, branch, recovery, and lifecycle state is rooted here.
pub struct RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) state: Arc<RuntimeWorldOwnerState<D, I, E, Ctx, T>>,
}

impl<D, I, E, Ctx, T> std::fmt::Debug for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeWorldOwnerRoot")
            .field("owner_identity", &self.owner_identity())
            .finish_non_exhaustive()
    }
}

pub(super) struct RuntimeWorldOwnerState<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) operation_control: super::RuntimeWorldOperationControlForTest,
    pub(super) owner_present: AtomicBool,
    pub(super) owner_identity: RuntimeWorldOwnerIdentity,
    pub(super) identities: Mutex<RuntimeWorldIdentityIssuer>,
    pub(crate) durable: Option<Arc<crate::durability::WorldDurableMediumBinding>>,
    pub(super) relational: Option<RelationalOwnerServicePorts>,
    pub(super) signal: RuntimeWorldSignalServices<D, I, E, Ctx, T>,
    pub(super) bridge: Option<RuntimeWorldCorrespondencePort>,
    pub(super) execution: super::execution::InstalledExecution,
    pub(super) clock: RuntimeWorldClock,
    pub(super) history: CompositeHistoryCatalog,
    pub(super) retention: RuntimeWorldRetentionOwner<D, I, T>,
    pub(super) branches: ProductBranchRegistry,
    pub(super) custody: crate::branch::OwnerCreatedComponentCustodyRegistry,
    pub(super) recovery: RecoveryCatalog,
    pub(super) bootstrap: Mutex<RuntimeWorldBootstrapState>,
    pub(super) close: Mutex<RuntimeWorldCloseContract>,
    pub(super) operation: RuntimeWorldOperationLedger,
    pub(super) publication_capacity: RuntimeWorldPublicationCapacityLedger,
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn new(
        inputs: RuntimeWorldOwnerInputs<D, I, E, Ctx, T>,
    ) -> Result<Self, RuntimeWorldIdentityExhaustion> {
        let construction = RuntimeWorldOwnerConstructionContract::new()?;
        Self::construct(inputs.into_installed(), construction, None)
    }

    pub(in crate::lifecycle) fn construct(
        inputs: RuntimeWorldInstalledOwnerInputs<D, I, E, Ctx, T>,
        construction: RuntimeWorldOwnerConstructionContract,
        durable: Option<Arc<crate::durability::WorldDurableMediumBinding>>,
    ) -> Result<Self, RuntimeWorldIdentityExhaustion> {
        let RuntimeWorldInstalledOwnerInputs {
            relational,
            signal,
            bridge,
            budgets,
            clock,
            execution,
        } = inputs;
        let owner_identity = construction.owner_identity();
        let identities = construction.into_issuer();
        let history = CompositeHistoryCatalog::new(
            owner_identity,
            crate::history::RuntimeWorldHistoryCatalogContract::installed(
                budgets.retained_composite_commits(),
                budgets.history_metadata_bytes(),
            ),
        );
        let retention = RuntimeWorldRetentionOwner::new(
            owner_identity,
            relational.basis_port(),
            signal.basis_port(),
            budgets.unique_exact_component_pins(),
            budgets.in_flight_pin_acquisition_reservations(),
            budgets.active_observations(),
        );
        let branches = ProductBranchRegistry::new(owner_identity, budgets.live_product_branches());
        let custody = crate::branch::OwnerCreatedComponentCustodyRegistry::new(
            budgets.owner_created_component_custody_records(),
        );
        let recovery = RecoveryCatalog::new_with_metadata(
            owner_identity,
            budgets.retained_product_unpublished_records(),
            budgets.retained_partial_metadata_bytes(),
        );
        let publication_capacity =
            RuntimeWorldPublicationCapacityLedger::new(budgets.active_publication_attempts());
        Ok(Self {
            state: Arc::new(RuntimeWorldOwnerState {
                operation_control: super::RuntimeWorldOperationControlForTest::default(),
                owner_present: AtomicBool::new(true),
                owner_identity,
                identities: Mutex::new(identities),
                durable,
                relational: Some(relational),
                signal,
                bridge: Some(bridge),
                execution,
                clock,
                history,
                retention,
                branches,
                custody,
                recovery,
                bootstrap: Mutex::new(RuntimeWorldBootstrapState::Unperformed),
                close: Mutex::new(RuntimeWorldCloseContract::open()),
                operation: RuntimeWorldOperationLedger::new(),
                publication_capacity,
            }),
        })
    }

    pub fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.state.owner_identity
    }

    pub(super) fn owner_is_present(&self) -> bool {
        self.state.owner_present.load(Ordering::Acquire)
    }

    pub(super) fn release_public_owner(&self) {
        let _bootstrap = self
            .state
            .bootstrap
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _operation = self
            .state
            .operation
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        self.state.owner_present.store(false, Ordering::Release);
    }
}

#[cfg(test)]
#[path = "owner_tests.rs"]
mod tests;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerState<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn relational_services(&self) -> &RelationalOwnerServicePorts {
        self.relational
            .as_ref()
            .expect("only the original serving World uses Native services")
    }
}
