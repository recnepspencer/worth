//! Prepared cold-safe fork ancestry staging and installation.
//!
//! Historical forks reconstruct any required truncated index, naming basis,
//! and group state before durable fork publication—whether the source is cold
//! or still holds full resident history (so a concurrent checkpoint ACK
//! clearing `full_history` mid-fork can never force post-durable cold reads).

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::lineage::groups::{BranchGroupState, GroupStateDenial};
use crate::lineage::index::PersistentNameBranchIndex;

use super::name_indexes::BranchNameIndex;
use super::naming_basis::NamingBasisUnavailable;
use super::LineageState;

/// Historical lineage state staged before durable fork publication.
#[derive(Debug)]
pub struct PreparedForkLineage {
    target: BranchId,
    source: BranchId,
    fork_commit_id: CommitId,
    group_state: Option<BranchGroupState>,
    name_index: Option<PersistentNameBranchIndex>,
}

fn map_group_denial(denial: GroupStateDenial) -> NamingBasisUnavailable {
    match denial {
        GroupStateDenial::StorageUnavailable(cause) => NamingBasisUnavailable::Storage(cause),
        _ => NamingBasisUnavailable::Unrecoverable,
    }
}

impl LineageState {
    /// Prepares a fork from `source` at `fork_commit_id` before durable
    /// publication, staging every historical component (`head/latest > cutoff`)
    /// regardless of `full_history` so cutover performs zero cold reads even if
    /// a concurrent checkpoint ACK clears `full_history` during durable I/O.
    pub(crate) fn prepare_fork_ancestry(
        &self,
        target: BranchId,
        source: BranchId,
        fork_commit_id: CommitId,
    ) -> Result<PreparedForkLineage, NamingBasisUnavailable> {
        if let Some(BranchNameIndex::Current(index)) = self.name_indexes.by_branch.get(&source)
            && index
                .retention_floor()
                .is_some_and(|floor| fork_commit_id < floor)
        {
            return Err(NamingBasisUnavailable::Unrecoverable);
        }
        let mut prepared = PreparedForkLineage {
            target,
            source,
            fork_commit_id,
            group_state: None,
            name_index: None,
        };
        if let Some(BranchNameIndex::Current(index)) =
            self.name_indexes.by_branch.get(&prepared.source)
            && index.head().is_some_and(|head| head > fork_commit_id)
        {
            let truncated = index
                .truncate_at(prepared.target.clone(), fork_commit_id)
                .map_err(|denial| NamingBasisUnavailable::from(&denial))?;
            prepared.name_index = Some(truncated);
        }
        if let Some(Ok(groups)) = self.group_states.by_branch.get(&prepared.source)
            && groups.latest_commit().is_some_and(|at| at > fork_commit_id)
        {
            let rebuilt = match self.full_history.as_ref() {
                Some(full) => {
                    self.rebuild_branch_group_state(full, &prepared.source, fork_commit_id)
                }
                None => self.rebuild_historical_group_state(&prepared.source, fork_commit_id),
            }
            .and_then(|rebuilt| {
                let link = groups.base().map(|b| b.forked(prepared.target.clone()));
                rebuilt
                    .rebase_over(groups, link)
                    .map_err(GroupStateDenial::StorageUnavailable)
            })
            .map_err(map_group_denial)?;
            prepared.group_state = Some(rebuilt);
        }
        Ok(prepared)
    }

    /// Installs a fork prepared by [`Self::prepare_fork_ancestry`] after durable
    /// publication, with zero cold reads.
    pub(crate) fn install_prepared_fork_ancestry(&mut self, prepared: PreparedForkLineage) {
        let PreparedForkLineage {
            target,
            source,
            fork_commit_id,
            group_state,
            name_index,
        } = prepared;
        self.record_fork_ancestry_metadata(target.clone(), source.clone(), fork_commit_id);
        self.bump_branch_generation(&target);
        match group_state {
            Some(groups) => {
                self.group_states
                    .by_branch
                    .insert(target.clone(), Ok(groups));
            }
            None => self.fork_group_state(&target, &source, fork_commit_id),
        }
        match name_index {
            Some(index) => {
                self.name_indexes
                    .by_branch
                    .insert(target.clone(), BranchNameIndex::Current(index));
            }
            None => self.fork_name_index(&target, &source, fork_commit_id),
        }
    }

    /// Registers `target` forked from `source` at `fork_commit_id`.
    pub(crate) fn register_fork_ancestry(
        &mut self,
        target: BranchId,
        source: BranchId,
        fork_commit_id: CommitId,
    ) {
        self.install_prepared_fork_ancestry(PreparedForkLineage {
            target,
            source,
            fork_commit_id,
            group_state: None,
            name_index: None,
        });
    }

}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as GEv;
    use crate::contracts::aggregate_identity::transition::group::delta::{
        AggregateGroupAssignment, AggregateGroupChange,
    };
    use crate::contracts::aggregate_identity::transition::group::model::{
        AggregateContinuityRule, AggregateGroupMembers, AggregateGroupScheme,
        AggregateGroupSchemeId, AggregateMemberRole,
    };
    use crate::contracts::aggregate_identity::transition::group::transition_result::AggregateGroupTransition;
    use crate::contracts::graph_member::NativeMemberKey;
    use crate::contracts::identity::data::primitives::{LineageId, PartitionId};
    use crate::contracts::identity::data::record_ids::EntityId;
    use crate::contracts::lineage::data::events::LineageEventKind;
    use crate::contracts::lineage::data::graph::assemble_lineage_node;
    use crate::contracts::lineage::data::naming::{
        PersistentNameHolder, PersistentNameIdentity, PersistentNameIdentityOwnerPort as _,
    };
    use crate::lineage::groups::{CommittedGroupTransition, GroupTransitionKind};
    use crate::lineage::naming_pages::tests::unnamed_retirement_and_group_log_carry::unnamed_event;
    use worth_foundational::facade::AggregateIdentity;

    #[test]
    fn resident_prepare_fork_stages_components_surviving_concurrent_ack_clearing_full_history() {
        let mut state = LineageState::empty();
        let (main, child) = (BranchId("main".into()), BranchId("hist-child".into()));
        let (e1, l1, l2) = (
            EntityId::new(PartitionId::main(), 1, 1),
            LineageId(1),
            LineageId(2),
        );
        state.record_node(assemble_lineage_node(l1, e1));
        state.record_node(assemble_lineage_node(
            l2,
            EntityId::new(PartitionId::main(), 2, 1),
        ));
        let ev = |id, c, kind, src, tgt| unnamed_event(id, &main, CommitId(c), kind, src, tgt);
        let scheme = AggregateGroupScheme::new(
            AggregateGroupSchemeId::new("test.s").unwrap(),
            AggregateContinuityRule::Anchor,
        );
        let mk_grp = |g: u8, retired: Option<u8>| {
            let gid = AggregateIdentity::from_bytes([g; 32]);
            let name = PersistentNameIdentity::from_minted_digest([g; 32]);
            let mut members = AggregateGroupMembers::new();
            members
                .insert(
                    name,
                    NativeMemberKey::Entity(e1),
                    AggregateMemberRole::Anchor,
                )
                .unwrap();
            let assign = AggregateGroupAssignment {
                identity: gid,
                anchor: name.into(),
                sources: Vec::new(),
                change: AggregateGroupChange::Born(members),
            };
            let ret: BTreeSet<_> = retired
                .map(|r| AggregateIdentity::from_bytes([r; 32]))
                .into_iter()
                .collect();
            let mut lin = BTreeSet::from([GEv::Born { group: gid }]);
            if let Some(r) = retired {
                lin.insert(GEv::Retired {
                    group: AggregateIdentity::from_bytes([r; 32]),
                });
            }
            CommittedGroupTransition {
                kind: GroupTransitionKind::Partition,
                scheme: scheme.clone(),
                transition: AggregateGroupTransition {
                    assignments: vec![assign],
                    retired: ret,
                    lineage: lin,
                },
            }
        };
        for (c, kind, src, tgt, g, ret) in [
            (1, LineageEventKind::Create, vec![], vec![l1], 1, None),
            (2, LineageEventKind::Create, vec![], vec![l2], 2, Some(1)),
            (3, LineageEventKind::Retire, vec![l1], vec![], 3, Some(2)),
        ] {
            let batch = [ev(c, c, kind, src, tgt)];
            state
                .install_recovered_event_batch(&batch, CommitId(c))
                .unwrap();
            state.record_group_lineage(&main, CommitId(c), vec![mk_grp(g, ret)]);
        }
        if let Some(BranchNameIndex::Current(src)) = state.name_indexes.by_branch.get_mut(&main) {
            src.advance_retention_floor(CommitId(2)).unwrap();
        }
        assert!(state.full_history.is_some());
        let prepared = state
            .prepare_fork_ancestry(child.clone(), main, CommitId(2))
            .unwrap();
        assert!(
            prepared.name_index.is_some()
                && prepared.group_state.is_some()
        );
        state.full_history = None;
        state.cold_naming = None;
        state.published_group_lineage.clear();
        state.install_prepared_fork_ancestry(prepared);
        let basis = state.authoritative_branch_naming_basis(&child);
        assert_eq!(
            basis.live_lineages.get(&PersistentNameHolder::Entity(e1)),
            Some(&l1)
        );
        let Some(BranchNameIndex::Current(idx)) = state.name_indexes.by_branch.get(&child) else {
            panic!("expected current child index");
        };
        assert_eq!(
            (idx.head(), idx.retention_floor()),
            (Some(CommitId(2)), Some(CommitId(2)))
        );
        let g1_key = crate::contracts::lineage::data::name_resolution::PersistentNameKey::Group(
            AggregateIdentity::from_bytes([1; 32]),
        );
        assert!(idx
            .resolve(
                g1_key,
                CommitId(1),
                &crate::contracts::lineage::data::name_resolution::PersistentNameSelection::Exact,
            )
            .result
            .is_err());
        assert!(matches!(
            state.group_states.by_branch.get(&child),
            Some(Ok(groups)) if groups.latest_commit() == Some(CommitId(2))
        ));
    }
}
