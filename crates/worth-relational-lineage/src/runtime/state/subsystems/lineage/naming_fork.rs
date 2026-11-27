//! Prepared cold-safe fork ancestry staging and installation.
//!
//! Historical forks reconstruct any required truncated index, naming basis,
//! and group state before durable fork publication while source coordination
//! holds off writers and retention-floor raises, so post-durable installation
//! performs zero cold reads.

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::lineage::groups::{BranchGroupState, GroupStateDenial};
use crate::lineage::index::{PersistentNameBranchIndex, PersistentNameIndexDenial};

use super::name_indexes::BranchNameIndex;
use super::naming_basis::{AuthoritativeBranchNamingState, NamingBasisUnavailable};
use super::naming_rebuild::{rebuild_branch_naming_state_for_branch, BranchNaming};
use super::LineageState;

/// Cold historical lineage state prepared before durable fork publication.
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
    /// publication, carrying any reconstructed cold historical state so
    /// installation performs zero cold reads.
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
        if self.full_history.is_some() {
            return Ok(prepared);
        }
        if let Some(BranchNameIndex::Current(index)) =
            self.name_indexes.by_branch.get(&prepared.source)
            && index.head().is_some_and(|head| head > fork_commit_id)
        {
            let truncated = index
                .truncate_at(prepared.target.clone(), fork_commit_id)
                .map_err(map_index_denial)?;
            prepared.name_index = Some(truncated);
        }
        if let Some(Ok(state)) = self.branch_naming_states.get(&prepared.source)
            && state
                .latest_event_commit()
                .is_some_and(|latest| latest > fork_commit_id)
        {
            let basis = if let Some(ref truncated) = prepared.name_index {
                AuthoritativeBranchNamingState::from_truncated_index(truncated, fork_commit_id)?
            } else {
                match self.name_indexes.by_branch.get(&prepared.source) {
                    Some(BranchNameIndex::Current(index)) => {
                        AuthoritativeBranchNamingState::from_truncated_index(index, fork_commit_id)?
                    }
                    Some(BranchNameIndex::Unavailable(denial)) => {
                        return Err(map_index_denial(denial.clone()));
                    }
                    None => return Err(NamingBasisUnavailable::Unrecoverable),
                }
            };
            prepared.naming_state = Some(basis);
        }
        if let Some(Ok(groups)) = self.group_states.by_branch.get(&prepared.source)
            && groups.latest_commit().is_some_and(|at| at > fork_commit_id)
        {
            let rebuilt = self
                .rebuild_historical_group_state(&prepared.source, fork_commit_id)
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
