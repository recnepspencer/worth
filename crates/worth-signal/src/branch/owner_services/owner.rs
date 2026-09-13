use super::owner_metadata::SignalOwnerMetadata;
use super::{
    SignalBranchCellState, SignalBranchExecutionCell, SignalBranchRegistry,
    SignalBranchRegistryDenial, SignalBranchRetirement, SignalOwnerAdmissionDenial,
    SignalOwnerLifecycleState, SignalOwnerOperationAdmission, SignalOwnerServiceCostSnapshot,
    SignalOwnerServiceCounters, SignalOwnerUnavailable,
};
use crate::branch::{SignalBranchBasisRegistry, SignalBranchRetentionRegistry};
use crate::logic::transaction::SignalOwnerPartition;
use crate::state::SignalBranchId;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Weak};
pub(super) mod basis;
mod basis_authority;
#[cfg(test)]
mod branch_incarnation_replacement;
pub(super) mod close_cleanup;
pub(super) mod fork_denials;
pub(super) mod fork_destination;
pub(super) mod fork_reservation;
mod inspection;
mod output_retention;
mod retention;
#[path = "owner/retention_preflight.rs"]
mod retention_preflight;
mod retirement_batch_execution;
mod retirement_batch_planning;
mod retirement_planning;
pub(super) mod retirement_reservation;
mod root;
use super::lifecycle_state::{SignalOwnerCloseCoordinator, SignalOwnerCloseDenial};
#[cfg(test)]
mod managed_reference_replacement_tests;
pub(crate) const DEFAULT_MAXIMUM_LIVE_SIGNAL_BRANCHES: usize = 4_096;
pub(crate) const DEFAULT_MAXIMUM_SIGNAL_BRANCH_RESERVATIONS: usize = 64;
const OWNER_CLOSE_BATCH_SIZE: usize = 64;
/// Why the current runtime cannot enter the independent owner-service posture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalOwnerServiceIssuanceDenial {
    EventSubscriberStateConfigured,
    ObservationRegistrationStateConfigured,
    ManagedQueueStateConfigured { bound_queue_count: u32 },
    LiveBranchCapacityExhausted { maximum_live_branches: usize },
    RetirementReceiptCapacityExhausted { maximum_retained_receipts: usize },
    ConditionalRetentionBudgetMismatch,
    DefinitionPublicationAlreadyIssued,
}

/// Sole strong owner state retained by a sealed, non-cloneable runtime root.
pub(crate) struct SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime_instance_id: u64,
    definition_basis: u64,
    next_branch_id: AtomicU64,
    lifecycle: Arc<SignalOwnerLifecycleState>,
    registry: Arc<SignalBranchRegistry<SignalBranchCellState<D, I, T>>>,
    basis_registry: SignalBranchBasisRegistry,
    retention: SignalBranchRetentionRegistry,
    pub(in crate::branch::owner_services) conditional_retention:
        Arc<super::conditional_execution::SignalConditionalRetentionLedger>,
    pub(in crate::branch::owner_services) conditional_temporal:
        super::conditional_execution::SignalConditionalTemporalRegistry,
    selected_branch_id: SignalBranchId,
    pub(super) metadata: SignalOwnerMetadata<D, I, T>,
    counters: Arc<SignalOwnerServiceCounters>,
}

pub(crate) use root::SignalOwnerRoot;

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn from_partition(
        runtime_instance_id: u64,
        definition_basis: u64,
        partition: SignalOwnerPartition<D, I, T>,
        basis_registry: SignalBranchBasisRegistry,
        conditional_budget: crate::runtime_policy::SignalConditionalEvaluationBudget,
        temporal_budget: crate::runtime_policy::SignalConditionalTemporalBudget,
    ) -> Arc<Self> {
        let (metadata, next_branch_id, retention, selected_branch_id, cells) =
            partition.into_parts();
        assert!(
            cells
                .iter()
                .any(|(handle, ..)| handle.id == selected_branch_id),
            "validated owner partition contains its selected branch"
        );
        let counters = Arc::new(SignalOwnerServiceCounters::default());
        let lifecycle = SignalOwnerLifecycleState::new(runtime_instance_id, Arc::clone(&counters));
        let lifecycle_identity = lifecycle.lifecycle_identity();
        let registry = Arc::new(SignalBranchRegistry::new(
            &lifecycle,
            DEFAULT_MAXIMUM_LIVE_SIGNAL_BRANCHES,
            DEFAULT_MAXIMUM_SIGNAL_BRANCH_RESERVATIONS,
        ));
        let owner = Arc::new(Self {
            runtime_instance_id,
            definition_basis,
            next_branch_id: AtomicU64::new(next_branch_id),
            lifecycle,
            registry,
            conditional_temporal:
                super::conditional_execution::SignalConditionalTemporalRegistry::new(
                    runtime_instance_id,
                    lifecycle_identity,
                ),
            basis_registry,
            retention,
            conditional_retention:
                super::conditional_execution::SignalConditionalRetentionLedger::new(
                    conditional_budget,
                    temporal_budget,
                ),
            selected_branch_id,
            metadata: SignalOwnerMetadata::new(metadata, runtime_instance_id, lifecycle_identity),
            counters,
        });
        let admission = owner
            .admit()
            .expect("a newly sealed Signal owner admits its canonical cells");
        for (handle, mut state, head_generation, restore_snapshot_id) in cells {
            state.seal_definition_custody(definition_basis);
            let branch_id = handle.id;
            let cell = owner
                .registry
                .reserve_named(&admission, branch_id, handle.name.clone())
                .and_then(|reservation| {
                    reservation.install(SignalBranchCellState::new(
                        handle,
                        runtime_instance_id,
                        definition_basis,
                        state,
                        head_generation,
                        restore_snapshot_id,
                    ))
                })
                .expect("validated owner partition installs each live branch exactly once");
            owner.basis_registry.rebind_cell_incarnation(
                runtime_instance_id,
                definition_basis,
                branch_id,
                0,
                cell.incarnation().get(),
            );
        }
        drop(admission);
        owner
    }

    pub(super) fn upgrade(weak: &Weak<Self>) -> Result<Arc<Self>, SignalOwnerUnavailable> {
        let owner = weak.upgrade().ok_or(SignalOwnerUnavailable)?;
        owner.counters.record_owner_upgrade_attempt();
        Ok(owner)
    }

    pub(super) fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub(super) fn definition_basis(&self) -> u64 {
        self.definition_basis
    }

    pub(super) fn lifecycle_identity(
        &self,
    ) -> super::lifecycle_state::SignalOwnerLifecycleIdentity {
        self.lifecycle.lifecycle_identity()
    }

    pub(super) fn lifecycle_observation(&self) -> super::SignalOwnerLifecycleObservation {
        self.lifecycle.observation()
    }

    #[cfg(test)]
    pub(super) fn cleanup_waiter_count(&self) -> usize {
        self.lifecycle.cleanup_waiter_count()
    }

    pub(super) fn admit(
        self: &Arc<Self>,
    ) -> Result<SignalOwnerOperationAdmission<'_>, SignalOwnerAdmissionDenial> {
        let close_coordinator: Arc<dyn SignalOwnerCloseCoordinator + '_> = self.clone();
        self.lifecycle
            .admit_with_close_coordinator(self.runtime_instance_id, close_coordinator)
    }

    pub(in crate::branch::owner_services) fn registry(
        &self,
    ) -> &Arc<SignalBranchRegistry<SignalBranchCellState<D, I, T>>> {
        &self.registry
    }

    pub(super) fn selected_branch_id(&self) -> SignalBranchId {
        self.selected_branch_id
    }

    pub(super) fn close(&self) -> Result<(), SignalOwnerCloseDenial> {
        self.lifecycle
            .begin_explicit_close(self.runtime_instance_id)?;
        self.retention.close_owner();
        self.conditional_retention.close();
        loop {
            self.finish_owner_close_cleanup();
            if self.lifecycle_observation() == super::SignalOwnerLifecycleObservation::Closed {
                return Ok(());
            }
            self.lifecycle.wait_for_cleanup_turn();
        }
    }

    fn request_close(&self) -> Result<(), SignalOwnerCloseDenial> {
        self.lifecycle.request_close(self.runtime_instance_id)?;
        self.retention.close_owner();
        self.conditional_retention.close();
        self.finish_owner_close_cleanup();
        Ok(())
    }

    pub(super) fn lookup_cell(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<
        Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
        SignalBranchRegistryDenial,
    > {
        self.registry.lookup(admission, branch_id)
    }

    pub(in crate::branch::owner_services) fn is_current_canonical_basis(
        &self,
        basis: &crate::branch::AdmittedSignalBranchBasis,
        branch_id: SignalBranchId,
        cell_incarnation: u64,
        observation: &crate::branch::SignalBranchObservation,
    ) -> bool {
        self.basis_registry.is_current_canonical_basis(
            self.runtime_instance_id,
            self.definition_basis,
            branch_id,
            cell_incarnation,
            observation,
            basis,
        )
    }

    pub(super) fn live_count(&self) -> usize {
        self.registry.live_count()
    }

    #[cfg(test)]
    pub(super) fn reservation_count(&self) -> usize {
        self.registry.reservation_count()
    }

    pub(super) fn begin_retirement<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<
        SignalBranchRetirement<'a, SignalBranchCellState<D, I, T>>,
        SignalBranchRegistryDenial,
    > {
        self.registry.begin_retirement(admission, branch_id)
    }

    pub(super) fn cost_snapshot(&self) -> SignalOwnerServiceCostSnapshot {
        self.counters.snapshot()
    }

    #[cfg(any(test, feature = "test-operation-control"))]
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "Phase 4 service operation tests consume this feature-only seam"
        )
    )]
    pub(in crate::branch::owner_services) fn operation_control(
        &self,
    ) -> super::operation_control::SignalOwnerOperationControl {
        self.lifecycle.operation_control()
    }

    pub(super) fn reach_operation_boundary(
        &self,
        boundary: super::operation_control::SignalOwnerOperationBoundary,
    ) {
        self.lifecycle.reach_operation_boundary(boundary);
    }
}
