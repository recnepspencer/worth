use std::sync::RwLockReadGuard;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity};
use crate::contracts::lineage::data::postures::LineageBasisView;
use worth_relational_storage::internal_port::storage::substrate::owned_state::RuntimeOwnedState;

use super::naming_basis::AuthoritativeLineageBasis;
use super::LineageState;

/// Zero-clone read-lock guard exposing O(log N) point lookups into one branch's
/// authoritative naming basis for commit finalization.
pub struct BranchNamingBasisGuard<'a> {
    guard: RwLockReadGuard<'a, LineageState>,
    branch_id: BranchId,
}

impl LineageBasisView for BranchNamingBasisGuard<'_> {
    fn live_holder(&self, id: &PersistentNameIdentity) -> Option<&PersistentNameHolder> {
        self.guard
            .branch_naming_state(&self.branch_id)
            .and_then(|state| state.live_holder(id))
    }

    fn live_lineage(&self, holder: &PersistentNameHolder) -> Option<LineageId> {
        self.guard
            .branch_naming_state(&self.branch_id)
            .and_then(|state| state.live_lineage(holder))
    }

    fn retired_lineage(&self, id: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)> {
        self.guard
            .branch_naming_state(&self.branch_id)
            .and_then(|state| state.retired_lineage(id))
    }

    fn holder_name(&self, holder: &PersistentNameHolder) -> Option<PersistentNameIdentity> {
        self.guard
            .branch_naming_state(&self.branch_id)
            .and_then(|state| state.holder_name(holder))
    }
}

/// Narrow read/fork-registration binding handed to `RelationalPreparationRuntime`
/// and `RelationalForkPort`.
#[derive(Debug)]
pub struct LineageNamingBasisBinding {
    state: RuntimeOwnedState<LineageState>,
}

impl Clone for LineageNamingBasisBinding {
    fn clone(&self) -> Self {
        Self {
            state: self.state.share(),
        }
    }
}

impl LineageNamingBasisBinding {
    pub(super) fn new(state: RuntimeOwnedState<LineageState>) -> Self {
        Self { state }
    }

    pub fn read_guard(&self, branch_id: BranchId) -> BranchNamingBasisGuard<'_> {
        BranchNamingBasisGuard {
            guard: self.state.read(),
            branch_id,
        }
    }

    pub fn register_fork_ancestry(
        &self,
        target_branch: BranchId,
        source_branch: BranchId,
        fork_commit_id: CommitId,
    ) {
        self.state
            .write()
            .register_fork_ancestry(target_branch, source_branch, fork_commit_id);
    }

    pub fn authoritative_basis(&self, branch_id: &BranchId) -> AuthoritativeLineageBasis {
        self.state
            .read()
            .authoritative_branch_naming_basis(branch_id)
    }
}
