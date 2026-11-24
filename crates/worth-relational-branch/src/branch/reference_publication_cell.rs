use crate::runtime::state::subsystems::history::history_costs::RelationalBranchSharingCostCounters;
use std::sync::Arc;

use crate::history::data::BranchId;

use super::reference::{RelationalBranchReferenceCell, RelationalBranchReferenceMutableState};

/// Stable, branch-local handle consumed by the publication port.
///
/// Clones address the same live reference state and coordination cell. Draft
/// branch cells remain detached until this handle installs their prepared
/// state after an exact-reference comparison succeeds.
#[derive(Clone, Debug)]
pub struct RelationalBranchPublicationCell {
    identity: crate::branch::identity::RelationalBranchIdentity,
    state: Arc<std::sync::Mutex<RelationalBranchReferenceMutableState>>,
    basis_registry: crate::branch::basis_registry::RelationalBranchBasisRegistry,
    coordination: Arc<super::coordination::RelationalBranchCoordinationCell>,
    head_retention: Arc<crate::history::retention::RelationalBranchHeadRetentionCell>,
    sharing_costs: crate::branch::sharing_cost_cell::RelationalBranchSharingCostCell,
}

pub struct RelationalBranchPublicationStateGuard<'cell> {
    cell: &'cell RelationalBranchPublicationCell,
    state: std::sync::MutexGuard<'cell, RelationalBranchReferenceMutableState>,
}

impl RelationalBranchReferenceCell {
    pub fn publication_cell(&self) -> RelationalBranchPublicationCell {
        RelationalBranchPublicationCell {
            identity: self.identity.clone(),
            state: Arc::clone(&self.state),
            basis_registry: self.basis_registry.clone(),
            coordination: Arc::clone(&self.coordination),
            head_retention: Arc::clone(&self.head_retention),
            sharing_costs: self.sharing_costs.clone(),
        }
    }
}

impl RelationalBranchPublicationCell {
    pub fn publication_state_locked_for_test(&self) -> bool {
        matches!(
            self.state.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        )
    }
    pub fn identity(&self) -> &crate::branch::identity::RelationalBranchIdentity {
        &self.identity
    }

    pub fn runtime_instance_id(&self) -> u64 {
        self.identity.runtime_instance_id()
    }

    pub fn branch_id(&self) -> &BranchId {
        self.identity.branch_id()
    }

    pub fn coordination(&self) -> &Arc<super::coordination::RelationalBranchCoordinationCell> {
        &self.coordination
    }

    pub fn enter_state(&self) -> RelationalBranchPublicationStateGuard<'_> {
        RelationalBranchPublicationStateGuard {
            cell: self,
            state: self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }

    pub fn currently_selects_root(
        &self,
        expected: &Arc<crate::branch::root::RelationalBranchRoot>,
    ) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .currently_selects_root(expected)
    }

    pub fn head_retention(
        &self,
    ) -> &Arc<crate::history::retention::RelationalBranchHeadRetentionCell> {
        &self.head_retention
    }

    pub fn register_basis(
        &self,
        basis: crate::branch::basis::AdmittedRelationalBranchBasis,
    ) -> Result<
        crate::branch::basis::AdmittedRelationalBranchBasis,
        crate::branch::basis_denial::RelationalBranchBasisDenial,
    > {
        self.basis_registry.register(basis)
    }

    pub fn record_sharing_cost(
        &self,
        update: impl FnOnce(&mut RelationalBranchSharingCostCounters),
    ) {
        self.sharing_costs.record(update);
    }

    pub fn record_sharing_cost_checked(
        &self,
        update: impl FnOnce(&mut RelationalBranchSharingCostCounters) -> Option<()>,
    ) {
        self.sharing_costs.record_checked_bytes(update);
    }

    pub fn sharing_costs(&self) -> RelationalBranchSharingCostCounters {
        self.sharing_costs.snapshot()
    }

    pub fn try_sharing_costs(
        &self,
    ) -> Result<
        RelationalBranchSharingCostCounters,
        crate::storage::substrate::StorageAllocationOverflow,
    > {
        self.sharing_costs.try_snapshot()
    }
}

impl RelationalBranchPublicationStateGuard<'_> {
    /// Compare against the actual admitted source while this original state
    /// guard is held. Do not call basis.is_current(), which reacquires it.
    pub fn matches_current_basis(
        &self,
        basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
    ) -> bool {
        if self.cell.identity() != basis.identity()
            || self.lifecycle_posture()
                != crate::branch::lifecycle::RelationalBranchLifecyclePosture::Live
            || self.observation() != basis.reference()
            || self.truth_version() != basis.truth_version()
        {
            return false;
        }
        self.state.root.as_ref().is_some_and(|root| {
            super::basis_axis_validation::require_current_descriptor_axes(
                basis.descriptor(),
                self.observation(),
                self.truth_version(),
                root,
            )
            .is_ok()
        })
    }
    pub fn selected_root(&self) -> Option<Arc<crate::branch::root::RelationalBranchRoot>> {
        self.state.root.as_ref().map(Arc::clone)
    }

    pub fn observation(
        &self,
    ) -> &crate::branch::reference::observation::RelationalBranchReferenceObservation {
        &self.state.observation
    }

    pub fn truth_version(&self) -> crate::branch::version::RelationalBranchVersion {
        self.state.truth_version
    }

    pub fn lifecycle_posture(&self) -> crate::branch::lifecycle::RelationalBranchLifecyclePosture {
        self.state.lifecycle_posture()
    }

    /// The observation archiving this branch would install.
    pub fn archived_observation(
        &self,
    ) -> Result<
        crate::branch::reference::observation::RelationalBranchReferenceObservation,
        crate::branch::reference::RelationalBranchCellDenial,
    > {
        self.state.archived_observation()
    }

    /// Install a prepared archive. Cannot fail.
    pub fn install_archive(
        &mut self,
        observation: crate::branch::reference::observation::RelationalBranchReferenceObservation,
    ) {
        self.state.install_archive(observation);
    }

    pub fn mark_deleting(&mut self) {
        self.state.mark_deleting();
    }

    pub fn snapshot_cell(&self) -> RelationalBranchReferenceCell {
        RelationalBranchReferenceCell {
            identity: self.cell.identity.clone(),
            state: Arc::new(std::sync::Mutex::new(self.state.clone())),
            basis_registry: self.cell.basis_registry.clone(),
            coordination: Arc::clone(&self.cell.coordination),
            head_retention: crate::history::retention::RelationalBranchHeadRetentionCell::fresh(),
            sharing_costs: self.cell.sharing_costs.detached_owner_snapshot(),
        }
    }

    pub fn replace_with(
        &mut self,
        prepared: RelationalBranchReferenceMutableState,
    ) -> Arc<crate::branch::root::RelationalBranchRoot> {
        let previous_root = self
            .state
            .root
            .take()
            .expect(/* Proof: The private live publication state is constructed with Some(root); this consuming transition is its only take. */ "live branch publication state carries its selected root");
        *self.state = prepared;
        previous_root
    }
}
