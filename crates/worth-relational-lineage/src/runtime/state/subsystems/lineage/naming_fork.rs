//! Prepared cold-safe fork ancestry staging and installation.
//!
//! Historical forks reconstruct any required truncated index, naming basis,
//! and group state before durable fork publication—whether the source is cold
//! or still holds full resident history (so a concurrent checkpoint ACK
//! clearing `full_history` mid-fork can never force post-durable cold reads).

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::lineage::groups::{BranchGroupState, GroupStateDenial};
use crate::lineage::index::{PersistentNameBranchIndex, PersistentNameIndexDenial};

use super::name_indexes::BranchNameIndex;
use super::naming_basis::{AuthoritativeBranchNamingState, NamingBasisUnavailable};
use super::naming_rebuild::{rebuild_branch_naming_state_for_branch, BranchNaming};
use super::LineageState;

/// Historical lineage state staged before durable fork publication.
#[derive(Debug)]
pub struct PreparedForkLineage {
    target: BranchId,
    source: BranchId,
    fork_commit_id: CommitId,
    group_state: Option<BranchGroupState>,
    name_index: Option<PersistentNameBranchIndex>,
    naming_state: Option<AuthoritativeBranchNamingState>,
}

fn map_index_denial(denial: PersistentNameIndexDenial) -> NamingBasisUnavailable {
    match denial {
        PersistentNameIndexDenial::StorageUnavailable(cause) => {
            NamingBasisUnavailable::Storage(cause)
        }
        _ => NamingBasisUnavailable::Unrecoverable,
    }
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
            naming_state: None,
        };
        if let Some(BranchNameIndex::Current(index)) =
            self.name_indexes.by_branch.get(&prepared.source)
            && index.head().is_some_and(|head| head > fork_commit_id)
        {
            let truncated = match self.full_history.as_ref() {
                Some(full) => {
                    match self.rebuild_branch_name_index(full, &prepared.source, fork_commit_id) {
                        BranchNameIndex::Current(rebuilt) => {
                            let mut child = rebuilt.rebranched(prepared.target.clone());
                            if let Some(floor) = index.retention_floor() {
                                child
                                    .advance_retention_floor(floor)
                                    .map_err(map_index_denial)?;
                            }
                            child
                        }
                        BranchNameIndex::Unavailable(denial) => {
                            return Err(map_index_denial(denial));
                        }
                    }
                }
                None => index
                    .truncate_at(prepared.target.clone(), fork_commit_id)
                    .map_err(map_index_denial)?,
            };
            prepared.name_index = Some(truncated);
        }
        if let Some(Ok(state)) = self.branch_naming_states.get(&prepared.source)
            && state
                .latest_event_commit()
                .is_some_and(|latest| latest > fork_commit_id)
        {
            let basis = match self.full_history.as_ref() {
                Some(full) => rebuild_branch_naming_state_for_branch(
                    full,
                    &prepared.source,
                    fork_commit_id,
                    &self.published_events,
                    &self.nodes,
                    &self.relation_lineage_holders,
                    &self.branch_forks,
                )?,
                None => match prepared.name_index.as_ref() {
                    Some(truncated) => AuthoritativeBranchNamingState::from_truncated_index(
                        truncated,
                        fork_commit_id,
                    )?,
                    None => match self.name_indexes.by_branch.get(&prepared.source) {
                        Some(BranchNameIndex::Current(index)) => {
                            AuthoritativeBranchNamingState::from_truncated_index(
                                index,
                                fork_commit_id,
                            )?
                        }
                        Some(BranchNameIndex::Unavailable(denial)) => {
                            return Err(map_index_denial(denial.clone()));
                        }
                        None => return Err(NamingBasisUnavailable::Unrecoverable),
                    },
                },
            };
            prepared.naming_state = Some(basis);
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
            naming_state,
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
        let seeded = match naming_state {
            Some(state) => Ok(state),
            None => self.seed_fork_naming_state(&target, &source, fork_commit_id),
        };
        self.branch_naming_states.insert(target, seeded);
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
            naming_state: None,
        });
    }

    fn seed_fork_naming_state(
        &self,
        target: &BranchId,
        source: &BranchId,
        fork_commit_id: CommitId,
    ) -> BranchNaming {
        match (
            self.branch_naming_states.get(source),
            self.full_history.as_ref(),
        ) {
            (Some(Ok(state)), _)
                if state
                    .latest_event_commit()
                    .is_none_or(|latest| latest <= fork_commit_id) =>
            {
                let link = state.base().map(|base| base.forked(target.clone()));
                Ok(state.fork(link))
            }
            (Some(Err(denial)), _) => Err(*denial),
            (_, Some(full)) => rebuild_branch_naming_state_for_branch(
                full,
                target,
                CommitId(u64::MAX),
                &self.published_events,
                &self.nodes,
                &self.relation_lineage_holders,
                &self.branch_forks,
            ),
            (None, None) => Ok(AuthoritativeBranchNamingState::over(None, None, None)),
            (Some(Ok(_)), None) => match self.name_indexes.by_branch.get(target) {
                Some(BranchNameIndex::Current(index)) => {
                    AuthoritativeBranchNamingState::from_truncated_index(index, fork_commit_id)
                }
                Some(BranchNameIndex::Unavailable(denial)) => Err(map_index_denial(denial.clone())),
                None => Err(NamingBasisUnavailable::Unrecoverable),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as GEv;
    use crate::contracts::aggregate_identity::transition::group::model::{
        AggregateContinuityRule, AggregateGroupAssignment, AggregateGroupMembers,
        AggregateGroupOrigin, AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
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
    use crate::lineage::naming_pages::tests::hardening_r4::unnamed_event;
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
            AggregateGroupSchemeId::new("s").unwrap(),
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
                anchor: name,
                sources: Vec::new(),
                members,
                origin: AggregateGroupOrigin::Born,
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
                && prepared.naming_state.is_some()
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
        use crate::contracts::lineage::data::name_resolution::{
            PersistentNameKey, PersistentNameSelection,
        };
        let g1_key = PersistentNameKey::Group(AggregateIdentity::from_bytes([1; 32]));
        assert!(idx
            .resolve(g1_key, CommitId(1), &PersistentNameSelection::Exact)
            .result
            .is_err());
        assert!(matches!(
            state.group_states.by_branch.get(&child),
            Some(Ok(groups)) if groups.latest_commit() == Some(CommitId(2))
        ));
    }
}
