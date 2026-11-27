//! A branch's exact group state at a past commit, for history selection.
//!
//! When the maintained state has applied nothing after `commit`, it is borrowed
//! in O(1) (`GroupStateAt::Head`). Otherwise it is replayed from published
//! group lineage along the branch's fork ancestry: a full-history state replays
//! its resident records through `commit`; a cold-backed state replays the
//! selected checkpoint's paged group log (one page resident at a time) and then
//! the records published since.

use std::collections::BTreeMap;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::groups::{BranchGroupState, GroupStateAt, GroupStateDenial};
use crate::lineage::naming_pages::codec::NamingCodec;
use crate::lineage::naming_pages::tables::{branch_key_digest, GROUP_LOG};

use super::LineageState;

impl LineageState {
    /// `branch`'s group state as of `commit`. Borrows the maintained state in
    /// O(1) when it has applied nothing after `commit`; otherwise replays
    /// canonical published group lineage through `commit`.
    pub(crate) fn group_state_at(
        &self,
        branch: &BranchId,
        commit: CommitId,
    ) -> Result<GroupStateAt<'_>, GroupStateDenial> {
        if let Some(Ok(state)) = self.group_states.by_branch.get(branch)
            && state.latest_commit().is_none_or(|at| at <= commit)
        {
            return Ok(GroupStateAt::Head(state));
        }
        self.rebuild_historical_group_state(branch, commit)
            .map(GroupStateAt::Rebuilt)
    }

    /// Replays `branch`'s group state through `commit` from full resident
    /// history or cold `GROUP_LOG` pages plus resident tail.
    pub(super) fn rebuild_historical_group_state(
        &self,
        branch: &BranchId,
        commit: CommitId,
    ) -> Result<BranchGroupState, GroupStateDenial> {
        if let Some(full) = self.full_history.as_ref() {
            return self.rebuild_branch_group_state(full, branch, commit);
        }
        let bounds: BTreeMap<_, _> = self.ancestry(branch, commit).into_iter().collect();
        let digest_bounds: BTreeMap<[u8; 32], CommitId> = bounds
            .iter()
            .map(|(b, &c)| (branch_key_digest(b), c))
            .collect();
        let newest = bounds.values().max().copied().unwrap_or(commit);
        let mut state = BranchGroupState::default();
        let mut through = None;
        if let Some(source) = &self.cold_naming {
            let store = source.store().ok_or(Cause::SourceUnavailable)?;
            through = store.state().group_log_through.map(CommitId);
            let mut refused = None;
            store.scan(&store.state().group_log, &[GROUP_LOG], |key, frame| {
                let ((_ordinal, at), owner_digest) =
                    <((u64, CommitId), [u8; 32])>::from_bytes(key.get(1..).ok_or(Cause::Damaged)?)?;
                if at > newest {
                    return Ok(false);
                }
                if digest_bounds
                    .get(&owner_digest)
                    .is_none_or(|bound| at > *bound)
                {
                    return Ok(true);
                }
                let transitions = crate::durability::naming::decode_group_lineage_frame(frame)
                    .map_err(|_| Cause::Damaged)?;
                match state.apply(at, &transitions) {
                    Ok(()) => Ok(true),
                    Err(denial) => {
                        refused = Some(denial);
                        Ok(false)
                    }
                }
            })?;
            if let Some(denial) = refused {
                return Err(denial);
            }
        }
        let tail_end = self
            .published_group_lineage
            .partition_point(|record| record.commit <= newest);
        for record in &self.published_group_lineage[..tail_end] {
            if through.is_some_and(|through| record.commit <= through) {
                continue;
            }
            if bounds
                .get(&record.branch)
                .is_some_and(|bound| record.commit <= *bound)
            {
                state.apply(record.commit, &record.transitions)?;
            }
        }
        Ok(state)
    }
}
