use std::cell::Cell;
use std::sync::RwLockReadGuard;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity};
use crate::contracts::lineage::data::postures::LineageBasisView;
use crate::lineage::tiered::NamingReadScope;
use worth_relational_storage::internal_port::storage::substrate::owned_state::RuntimeOwnedState;

use super::naming_basis::{
    AuthoritativeBranchNamingState, AuthoritativeLineageBasis, NamingBasisUnavailable,
};
use super::LineageState;

/// Zero-clone read-lock guard exposing `O(log N)` point lookups into one
/// branch's authoritative naming basis for commit finalization.
///
/// Rows faulted from cold pages live in the guard's own read scope. A failed
/// fault or an unavailable basis reads as absent and latches; every caller
/// settles the guard after its reads and refuses when it latched, so a
/// damaged page never reads as an absent name.
pub struct BranchNamingBasisGuard<'a> {
    guard: RwLockReadGuard<'a, LineageState>,
    branch_id: BranchId,
    scope: NamingReadScope,
    unavailable: Cell<Option<NamingBasisUnavailable>>,
}

impl BranchNamingBasisGuard<'_> {
    fn state(&self) -> Option<&AuthoritativeBranchNamingState> {
        match self.guard.branch_naming_state(&self.branch_id)? {
            Ok(state) => Some(state),
            Err(denial) => {
                self.latch(*denial);
                None
            }
        }
    }

    fn latch(&self, denial: NamingBasisUnavailable) {
        if self.unavailable.get().is_none() {
            self.unavailable.set(Some(denial));
        }
    }

    /// Settles every read made through this guard: `Ok` only when no fault
    /// failed and the basis was available.
    pub fn settle(&self) -> Result<(), NamingBasisUnavailable> {
        if let Some(denial) = self.unavailable.get() {
            return Err(denial);
        }
        self.scope.settle().map_err(NamingBasisUnavailable::Storage)
    }

    /// The guard's read scope, for views built over its name index.
    pub fn scope(&self) -> &NamingReadScope {
        &self.scope
    }
}

impl LineageBasisView for BranchNamingBasisGuard<'_> {
    fn live_holder(&self, id: &PersistentNameIdentity) -> Option<&PersistentNameHolder> {
        self.state()?.live_holder_in(id, &self.scope)
    }

    fn live_lineage(&self, holder: &PersistentNameHolder) -> Option<LineageId> {
        self.state()?.live_lineage_in(holder, &self.scope)
    }

    fn retired_lineage(&self, id: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)> {
        self.state()?.retired_lineage_in(id, &self.scope)
    }

    fn holder_name(&self, holder: &PersistentNameHolder) -> Option<PersistentNameIdentity> {
        self.state()?.holder_name_in(holder, &self.scope)
    }
}

impl BranchNamingBasisGuard<'_> {
    /// The guarded branch's derived name index under the same read lock;
    /// `None` before its first named lineage. Read it through
    /// [`Self::scope`] and settle the guard afterwards.
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
        self.state()?.latest_event_commit()
    }

    /// The guarded branch's group state; `None` before its first group
    /// lineage, and a refusal while it is unavailable.
    pub fn group_state(
        &self,
    ) -> Result<
        Option<crate::lineage::groups::GroupStateRead<'_>>,
        &crate::lineage::groups::GroupStateDenial,
    > {
        Ok(self
            .guard
            .branch_group_state(&self.branch_id)?
            .map(|state| state.read(&self.scope)))
    }

    /// `branch`'s group state exactly as of `commit`: the maintained state
    /// borrowed in O(1) at its head, else rebuilt from canonical published
    /// group lineage through `commit` (see `LineageState::group_state_at`).
    pub fn group_state_at(
        &self,
        branch: &BranchId,
        commit: CommitId,
    ) -> Result<crate::lineage::groups::GroupStateAt<'_>, crate::lineage::groups::GroupStateDenial>
    {
        self.guard.group_state_at(branch, commit)
    }

    /// Probes every cold naming-basis and derived name-index row that
    /// settlement will touch for `events`, `entity_bindings`,
    /// `relation_bindings` and `group_transitions`, latching any storage
    /// failure into this guard so preparation refuses before durable append.
    pub fn preflight_commit_settlement(
        &self,
        events: &[crate::contracts::lineage::data::events::LineageEventRecord],
        entity_bindings: &[(
            crate::contracts::identity::data::record_ids::EntityId,
            LineageId,
        )],
        relation_bindings: &[(
            crate::contracts::identity::data::record_ids::RelationId,
            LineageId,
        )],
        group_transitions: &[crate::lineage::groups::CommittedGroupTransition],
    ) {
        if let Some(state) = self.state() {
            state.preflight_events(events, &self.scope);
        }
        match self.guard.name_indexes.read(&self.branch_id) {
            Ok(Some(index)) => index.preflight_commit(
                events,
                entity_bindings,
                relation_bindings,
                group_transitions,
                &self.scope,
            ),
            Ok(None) => {}
            Err(crate::lineage::index::PersistentNameIndexDenial::StorageUnavailable(cause)) => {
                self.latch(NamingBasisUnavailable::Storage(*cause));
            }
            Err(_) => {
                self.latch(NamingBasisUnavailable::Unrecoverable);
            }
        }
    }

    /// The holder and name `lineage_id` carried on the guarded branch.
    pub fn holder_and_name_for_lineage(
        &self,
        lineage_id: LineageId,
    ) -> Option<(PersistentNameHolder, PersistentNameIdentity)> {
        match self
            .guard
            .holder_and_name_for_lineage(&self.branch_id, lineage_id)
        {
            Ok(found) => found,
            Err(denial) => {
                self.latch(denial);
                None
            }
        }
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
            scope: NamingReadScope::new(),
            unavailable: Cell::new(None),
        }
    }

    pub fn preflight_fork_ancestry(
        &self,
        source_branch: &BranchId,
        fork_commit_id: CommitId,
    ) -> Result<(), NamingBasisUnavailable> {
        self.state
            .read()
            .preflight_fork_ancestry(source_branch, fork_commit_id)
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

    /// Complexity: O(N) full scan; audits and courts only.
    pub fn authoritative_basis(&self, branch_id: &BranchId) -> AuthoritativeLineageBasis {
        self.state
            .read()
            .authoritative_branch_naming_basis(branch_id)
    }
}
