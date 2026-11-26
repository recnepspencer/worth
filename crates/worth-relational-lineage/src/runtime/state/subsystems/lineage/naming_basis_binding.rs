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

impl BranchNamingBasisGuard<'_> {
    /// The guarded branch's derived name index under the same read lock;
    /// `None` before its first named lineage.
    pub fn name_index(
        &self,
    ) -> Result<
        Option<&crate::lineage::index::PersistentNameBranchIndex>,
        &crate::lineage::index::PersistentNameIndexDenial,
    > {
        self.guard.name_indexes.read(&self.branch_id)
    }

    /// The latest commit whose lineage events this branch's naming basis
    /// holds; `None` before its first. A basis read for a proposal is only
    /// that proposal's basis while this is at or before the proposal's commit.
    pub fn latest_event_commit(&self) -> Option<CommitId> {
        self.guard
            .branch_naming_state(&self.branch_id)
            .and_then(|state| state.latest_event_commit())
    }

    /// The guarded branch's group state; `None` before its first group
    /// lineage, and a refusal while it is unavailable.
    pub fn group_state(
        &self,
    ) -> Result<
        Option<&crate::lineage::groups::BranchGroupState>,
        &crate::lineage::groups::GroupStateDenial,
    > {
        self.guard.branch_group_state(&self.branch_id)
    }

    /// `branch`'s group state exactly as of `commit`, rebuilt from its
    /// canonical published group lineage.
    ///
    /// Complexity: O(published group lineage along the ancestry).
    pub fn group_state_at(
        &self,
        branch: &BranchId,
        commit: CommitId,
    ) -> Result<crate::lineage::groups::BranchGroupState, crate::lineage::groups::GroupStateDenial>
    {
        self.guard.rebuild_branch_group_state(branch, commit)
    }

    /// The holder and name `lineage_id` carried on the guarded branch.
    pub fn holder_and_name_for_lineage(
        &self,
        lineage_id: LineageId,
    ) -> Option<(PersistentNameHolder, PersistentNameIdentity)> {
        self.guard
            .holder_and_name_for_lineage(&self.branch_id, lineage_id)
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
