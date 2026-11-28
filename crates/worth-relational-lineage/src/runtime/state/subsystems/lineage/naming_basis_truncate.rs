//! Truncated-index derivation of [`AuthoritativeBranchNamingState`] for cold
//! retained forks.

use std::collections::{BTreeMap, BTreeSet};

use super::naming_basis::{AuthoritativeBranchNamingState, NamingBasisUnavailable};
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameTarget};
use crate::contracts::lineage::data::naming::PersistentNameHolder;
use crate::lineage::index::truncate::IndexTruncationDiff;
use crate::lineage::index::{
    LineageHolder, NameState, PersistentNameBranchIndex, PersistentNameIndexDenial,
};

fn to_holder(h: LineageHolder) -> PersistentNameHolder {
    match h {
        LineageHolder::Entity(e) => PersistentNameHolder::Entity(e),
        LineageHolder::Relation(r) => PersistentNameHolder::Relation(r),
    }
}

fn to_index_holder(h: &PersistentNameHolder) -> Option<LineageHolder> {
    match *h {
        PersistentNameHolder::Entity(e) => Some(LineageHolder::Entity(e)),
        PersistentNameHolder::Relation(r) => Some(LineageHolder::Relation(r)),
        _ => None,
    }
}

impl AuthoritativeBranchNamingState {
    pub(super) fn count_diff_against_bounded(
        &self,
        base: &Self,
        has_cold_base: bool,
        budget: usize,
        visited: &mut usize,
    ) -> Option<usize> {
        let mut total = 0usize;
        macro_rules! step {
            ($field:ident) => {
                let rem = budget.saturating_sub(total);
                total += self.$field.count_diff_against_bounded(
                    &base.$field,
                    has_cold_base,
                    rem,
                    visited,
                )?;
                if total > budget {
                    return None;
                }
            };
        }
        step!(lineage_holders);
        step!(holder_names);
        step!(live_holders);
        step!(live_lineages);
        step!(retired_lineages);
        Some(total)
    }

    /// Truncates `source` to `fork_commit` over a shared fork of `source`'s cold
    /// base / `CowMap` spine, touching only keys recorded in `diff`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn truncate_from_source(
        source: &Self,
        source_index: &PersistentNameBranchIndex,
        truncated: &PersistentNameBranchIndex,
        diff: &IndexTruncationDiff,
        target: &BranchId,
        fork_commit: CommitId,
        event_bounds: Option<(Option<CommitId>, Option<CommitId>)>,
    ) -> Result<Self, NamingBasisUnavailable> {
        let mut state = source.fork(source.base().map(|b| b.forked(target.clone())));
        for &lineage in &diff.dropped_lineages {
            state.lineage_holders.remove_blind(&lineage);
        }
        let mut affected_holders: BTreeSet<PersistentNameHolder> = diff
            .changed_holders
            .iter()
            .copied()
            .map(to_holder)
            .collect();
        for &key in &diff.changed_names {
            let (subject, name_id) = match key {
                PersistentNameKey::Entity(id) => (LineageSubject::Entity, id),
                PersistentNameKey::Relation(id) => (LineageSubject::Relation, id),
                PersistentNameKey::Group(_) => continue,
            };
            if let Some(h) = source
                .live_holders
                .get(&name_id)
                .map_err(NamingBasisUnavailable::Storage)?
            {
                affected_holders.insert(h.into_owned());
            }
            if let Some(entry) = source_index
                .names
                .get(&key)
                .map_err(NamingBasisUnavailable::Storage)?
            {
                for &lineage in entry.lineage_commits.keys() {
                    if let Some(h) = source
                        .lineage_holders
                        .get(&lineage)
                        .map_err(NamingBasisUnavailable::Storage)?
                    {
                        affected_holders.insert(h.into_owned());
                    }
                }
            }
            match truncated
                .names
                .get(&key)
                .map_err(NamingBasisUnavailable::Storage)?
            {
                None => {
                    state.live_holders.remove_blind(&name_id);
                    state.retired_lineages.remove_blind(&name_id);
                }
                Some(entry) => {
                    for &lineage in entry.lineage_commits.keys() {
                        if let Some(h) = source
                            .lineage_holders
                            .get(&lineage)
                            .map_err(NamingBasisUnavailable::Storage)?
                        {
                            affected_holders.insert(h.into_owned());
                        }
                    }
                    match entry.last_state() {
                        (_, NameState::Live(live)) => {
                            let target_pair = match live.target {
                                PersistentNameTarget::Entity {
                                    entity, lineage, ..
                                } => Some((PersistentNameHolder::Entity(entity), lineage)),
                                PersistentNameTarget::Relation {
                                    relation, lineage, ..
                                } => Some((PersistentNameHolder::Relation(relation), lineage)),
                                PersistentNameTarget::Group { .. } => None,
                            };
                            if let Some((holder, lineage)) = target_pair {
                                affected_holders.insert(holder.clone());
                                state.lineage_holders.insert(lineage, holder.clone());
                                state.holder_names.insert(holder.clone(), name_id);
                                state.live_holders.insert(name_id, holder.clone());
                                state.live_lineages.insert(holder, lineage);
                                state.retired_lineages.remove_blind(&name_id);
                            }
                        }
                        (ended_at, NameState::Ended) => {
                            state.live_holders.remove_blind(&name_id);
                            if let Some(last_lineage) = entry.last_live_lineage() {
                                state
                                    .retired_lineages
                                    .insert(name_id, (subject, last_lineage, ended_at));
                            } else {
                                state.retired_lineages.remove_blind(&name_id);
                            }
                        }
                    }
                }
            }
        }
        for holder in affected_holders {
            let Some(idx_holder) = to_index_holder(&holder) else {
                continue;
            };
            match truncated
                .holder_epochs_for_truncation(&idx_holder)
                .map_err(NamingBasisUnavailable::Storage)?
            {
                None => {
                    state.live_lineages.remove_blind(&holder);
                    state.holder_names.remove_blind(&holder);
                }
                Some(epochs) => {
                    let mut latest_name = None;
                    for &(_, lineage) in epochs.0.keys() {
                        state.lineage_holders.insert(lineage, holder.clone());
                        if let Some(name_key) = truncated
                            .lineage_name_for_truncation(&lineage)
                            .map_err(NamingBasisUnavailable::Storage)?
                        {
                            match name_key {
                                PersistentNameKey::Entity(id) | PersistentNameKey::Relation(id) => {
                                    latest_name = Some(id)
                                }
                                PersistentNameKey::Group(_) => {}
                            }
                        }
                    }
                    match latest_name {
                        Some(id) => state.holder_names.insert(holder.clone(), id),
                        None => state.holder_names.remove_blind(&holder),
                    }
                    match epochs.latest() {
                        Some((_, lineage, ended_at))
                            if ended_at.is_none_or(|end| end > fork_commit) =>
                        {
                            state.live_lineages.insert(holder, lineage);
                        }
                        _ => state.live_lineages.remove_blind(&holder),
                    }
                }
            }
        }
        if let Some((earliest, latest)) = event_bounds {
            state.earliest_event_commit = earliest;
            state.latest_event_commit = latest;
        } else if truncated.head().is_some() {
            state.earliest_event_commit = source
                .earliest_event_commit()
                .filter(|&c| c <= fork_commit)
                .or_else(|| truncated.earliest_commit().map(|c| c.min(fork_commit)));
            state.latest_event_commit = source
                .latest_event_commit()
                .filter(|&c| c <= fork_commit)
                .or_else(|| truncated.head().map(|c| c.min(fork_commit)));
        } else {
            state.earliest_event_commit = None;
            state.latest_event_commit = None;
        }
        Ok(state)
    }

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
                && let Some((_, lineage, ended_at)) = epochs.latest()
                && ended_at.is_none_or(|end| end > fork_commit)
            {
                state.live_lineages.insert(holder, lineage);
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
