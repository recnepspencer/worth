use std::sync::Arc;

use crate::history::data::BranchId;

use super::reference::{RelationalBranchReferenceCell, RelationalBranchReferenceMutableState};

/// Stable, branch-local handle consumed by the publication port.
///
/// Clones address the same live reference state and coordination cell. Draft
/// branch cells remain detached until this handle installs their prepared
/// state after an exact-reference comparison succeeds.
#[derive(Clone, Debug)]
pub(crate) struct RelationalBranchPublicationCell {
    identity: super::RelationalBranchIdentity,
    state: Arc<std::sync::Mutex<RelationalBranchReferenceMutableState>>,
    basis_registry: super::RelationalBranchBasisRegistry,
    coordination: Arc<super::coordination::RelationalBranchCoordinationCell>,
    head_retention: Arc<crate::history::retention::RelationalBranchHeadRetentionCell>,
    sharing_costs: super::RelationalBranchSharingCostCell,
}

pub(crate) struct RelationalBranchPublicationStateGuard<'cell> {
    cell: &'cell RelationalBranchPublicationCell,
    state: std::sync::MutexGuard<'cell, RelationalBranchReferenceMutableState>,
}

impl RelationalBranchReferenceCell {
    pub(crate) fn publication_cell(&self) -> RelationalBranchPublicationCell {
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
    pub(crate) fn identity(&self) -> &super::RelationalBranchIdentity {
        &self.identity
    }

    pub(crate) fn runtime_instance_id(&self) -> u64 {
        self.identity.runtime_instance_id()
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        self.identity.branch_id()
    }

    pub(crate) fn coordination(
        &self,
    ) -> &Arc<super::coordination::RelationalBranchCoordinationCell> {
        &self.coordination
    }

    pub(crate) fn enter_state(&self) -> RelationalBranchPublicationStateGuard<'_> {
        RelationalBranchPublicationStateGuard {
            cell: self,
            state: self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }

    pub(crate) fn currently_selects_root(
        &self,
        expected: &Arc<super::RelationalBranchRoot>,
    ) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .currently_selects_root(expected)
    }

    pub(crate) fn head_retention(
        &self,
    ) -> &Arc<crate::history::retention::RelationalBranchHeadRetentionCell> {
        &self.head_retention
    }

    pub(crate) fn register_basis(
        &self,
        basis: super::AdmittedRelationalBranchBasis,
    ) -> Result<super::AdmittedRelationalBranchBasis, super::RelationalBranchBasisDenial> {
        self.basis_registry.register(basis)
    }

    pub(crate) fn record_sharing_cost(
        &self,
        update: impl FnOnce(&mut crate::runtime::RelationalBranchSharingCostCounters),
    ) {
        self.sharing_costs.record(update);
    }

    pub(crate) fn sharing_costs(&self) -> crate::runtime::RelationalBranchSharingCostCounters {
        self.sharing_costs.snapshot()
    }
}

impl RelationalBranchPublicationStateGuard<'_> {
    pub(crate) fn selected_root(&self) -> Option<Arc<super::RelationalBranchRoot>> {
        self.state.root.as_ref().map(Arc::clone)
    }

    pub(crate) fn observation(&self) -> &super::RelationalBranchReferenceObservation {
        &self.state.observation
    }

    pub(crate) fn truth_version(&self) -> super::RelationalBranchVersion {
        self.state.truth_version
    }

    pub(crate) fn lifecycle_posture(&self) -> super::RelationalBranchLifecyclePosture {
        self.state.lifecycle_posture()
    }

    /// The observation archiving this branch would install.
    pub(crate) fn archived_observation(
        &self,
    ) -> Result<super::RelationalBranchReferenceObservation, super::RelationalBranchCellDenial>
    {
        self.state.archived_observation()
    }

    /// Install a prepared archive. Cannot fail.
    pub(crate) fn install_archive(
        &mut self,
        observation: super::RelationalBranchReferenceObservation,
    ) {
        self.state.install_archive(observation);
    }

    pub(crate) fn mark_deleting(&mut self) {
        self.state.mark_deleting();
    }

    pub(crate) fn snapshot_cell(&self) -> RelationalBranchReferenceCell {
        RelationalBranchReferenceCell {
            identity: self.cell.identity.clone(),
            state: Arc::new(std::sync::Mutex::new(self.state.clone())),
            basis_registry: self.cell.basis_registry.clone(),
            coordination: Arc::clone(&self.cell.coordination),
            head_retention: crate::history::retention::RelationalBranchHeadRetentionCell::fresh(),
            sharing_costs: self.cell.sharing_costs.detached_owner_snapshot(),
        }
    }

    pub(crate) fn replace_with(
        &mut self,
        prepared: RelationalBranchReferenceMutableState,
    ) -> Arc<super::RelationalBranchRoot> {
        let previous_root = self
            .state
            .root
            .take()
            .expect("live branch publication state carries its selected root");
        *self.state = prepared;
        previous_root
    }
}
