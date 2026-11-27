//! Truncated-index derivation of [`AuthoritativeBranchNamingState`] for cold
//! retained forks.

use std::collections::{BTreeMap, BTreeSet};

use super::naming_basis::{AuthoritativeBranchNamingState, NamingBasisUnavailable};
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameTarget};
use crate::contracts::lineage::data::naming::PersistentNameHolder;
use crate::lineage::index::{
    LineageHolder, NameState, PersistentNameBranchIndex, PersistentNameIndexDenial,
};

impl AuthoritativeBranchNamingState {
    /// Derives a branch's authoritative naming basis at `fork_commit` directly
    /// from a truncated `PersistentNameBranchIndex` (for cold retained forks).
    pub(crate) fn from_truncated_index(
        index: &PersistentNameBranchIndex,
        fork_commit: CommitId,
    ) -> Result<Self, NamingBasisUnavailable> {
        let (entries, holders, holder_epochs) =
            index
                .export_entries_for_naming_basis()
                .map_err(|denial| match denial {
                    PersistentNameIndexDenial::StorageUnavailable(cause) => {
                        NamingBasisUnavailable::Storage(cause)
                    }
                    _ => NamingBasisUnavailable::Unrecoverable,
                })?;
        let mut state = Self::over(None, None, None);
        let to_holder = |h: LineageHolder| match h {
            LineageHolder::Entity(e) => PersistentNameHolder::Entity(e),
            LineageHolder::Relation(r) => PersistentNameHolder::Relation(r),
        };
        let mut by_lineage = BTreeMap::new();
        for (lineage, holder) in holders {
            let holder = to_holder(holder);
            state.lineage_holders.insert(lineage, holder.clone());
            by_lineage.insert(lineage, holder);
        }
        let mut named_holders_seen = BTreeSet::new();
        for (key, entry) in entries {
            let (subject, name_id) = match key {
                PersistentNameKey::Entity(id) => (LineageSubject::Entity, id),
                PersistentNameKey::Relation(id) => (LineageSubject::Relation, id),
                PersistentNameKey::Group(_) => continue,
            };
            for &lineage in entry.lineage_commits.keys() {
                if let Some(holder) = by_lineage.get(&lineage) {
                    state.holder_names.insert(holder.clone(), name_id);
                    named_holders_seen.insert(holder.clone());
                }
            }
            match entry.last_state() {
                (_, NameState::Live(live)) => {
                    let Some((holder, lineage)) = (match live.target {
                        PersistentNameTarget::Entity {
                            entity, lineage, ..
                        } => Some((PersistentNameHolder::Entity(entity), lineage)),
                        PersistentNameTarget::Relation {
                            relation, lineage, ..
                        } => Some((PersistentNameHolder::Relation(relation), lineage)),
                        PersistentNameTarget::Group { .. } => None,
                    }) else {
                        continue;
                    };
                    state.lineage_holders.insert(lineage, holder.clone());
                    state.holder_names.insert(holder.clone(), name_id);
                    state.live_holders.insert(name_id, holder.clone());
                    state.live_lineages.insert(holder.clone(), lineage);
                    named_holders_seen.insert(holder);
                }
                (ended_at, NameState::Ended) => {
                    if let Some(last_lineage) = entry.last_live_lineage() {
                        state
                            .retired_lineages
                            .insert(name_id, (subject, last_lineage, ended_at));
                    }
                }
            }
        }
        for (holder, epochs) in holder_epochs {
            let holder = to_holder(holder);
            if !named_holders_seen.contains(&holder)
                && let Some(last) = epochs.last()
                && last.ended_at.is_none_or(|end| end > fork_commit)
            {
                state.live_lineages.insert(holder, last.lineage);
            }
        }
        if state.lineage_holders.has_delta()
            || state.holder_names.has_delta()
            || state.live_holders.has_delta()
            || state.live_lineages.has_delta()
            || state.retired_lineages.has_delta()
        {
            state.earliest_event_commit = index.earliest_commit().map(|c| c.min(fork_commit));
            state.latest_event_commit = index.head().map(|c| c.min(fork_commit));
        }
        Ok(state)
    }
}
