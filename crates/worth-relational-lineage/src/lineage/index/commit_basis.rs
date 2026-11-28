//! The commit-finalization projection over the canonical branch index.
use std::collections::BTreeMap;

use super::{LineageHolder, NameEntry, NameState, PersistentNameBranchIndex};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameTarget};
use crate::contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::tiered::NamingReadScope;
use crate::runtime::state::subsystems::lineage::naming_basis::AuthoritativeLineageBasis;

fn native_holder(holder: &PersistentNameHolder) -> Option<LineageHolder> {
    match holder {
        PersistentNameHolder::Entity(entity) => Some(LineageHolder::Entity(*entity)),
        PersistentNameHolder::Relation(relation) => Some(LineageHolder::Relation(*relation)),
        _ => None,
    }
}

fn holder_of_target(target: &PersistentNameTarget) -> Option<PersistentNameHolder> {
    match target {
        PersistentNameTarget::Entity { entity, .. } => Some(PersistentNameHolder::Entity(*entity)),
        PersistentNameTarget::Relation { relation, .. } => {
            Some(PersistentNameHolder::Relation(*relation))
        }
        PersistentNameTarget::Group { .. } => None,
    }
}

fn subject_and_name(key: PersistentNameKey) -> Option<(LineageSubject, PersistentNameIdentity)> {
    match key {
        PersistentNameKey::Entity(name) => Some((LineageSubject::Entity, name)),
        PersistentNameKey::Relation(name) => Some((LineageSubject::Relation, name)),
        PersistentNameKey::Group(_) => None,
    }
}

impl PersistentNameBranchIndex {
    fn name_entries<'a>(
        &'a self,
        name: PersistentNameIdentity,
        scope: &'a NamingReadScope,
    ) -> impl Iterator<Item = (PersistentNameKey, &'a NameEntry)> {
        [
            PersistentNameKey::Entity(name),
            PersistentNameKey::Relation(name),
        ]
        .into_iter()
        .filter_map(|key| self.entry_in(&key, scope).map(|entry| (key, entry)))
    }

    pub(crate) fn live_holder_in(
        &self,
        name: PersistentNameIdentity,
        scope: &NamingReadScope,
    ) -> Option<PersistentNameHolder> {
        self.name_entries(name, scope)
            .filter_map(|(_, entry)| {
                let (at, state) = entry.last_state();
                match state {
                    NameState::Live(live) => Some((at, holder_of_target(&live.target)?)),
                    NameState::Ended => None,
                }
            })
            .max_by_key(|(at, _)| *at)
            .map(|(_, holder)| holder)
    }

    pub(crate) fn retired_lineage_in(
        &self,
        name: PersistentNameIdentity,
        scope: &NamingReadScope,
    ) -> Option<(LineageSubject, LineageId)> {
        let (key, entry) = self
            .name_entries(name, scope)
            .max_by_key(|(_, entry)| entry.last_state().0)?;
        retired_lineage(key, entry)
    }

    pub(crate) fn live_lineage_in(
        &self,
        holder: &PersistentNameHolder,
        scope: &NamingReadScope,
    ) -> Option<LineageId> {
        let (_, lineage, ended_at) = self
            .holder_lineages
            .get_in(&native_holder(holder)?, scope)?
            .latest()?;
        ended_at.is_none().then_some(lineage)
    }

    pub(crate) fn holder_name_in(
        &self,
        holder: &PersistentNameHolder,
        scope: &NamingReadScope,
    ) -> Option<PersistentNameIdentity> {
        let lineage = self.latest_holder_lineage_in(native_holder(holder)?, scope)?;
        subject_and_name(self.name_of_lineage(lineage, scope)?).map(|(_, name)| name)
    }

    pub(crate) fn holder_and_name_in(
        &self,
        lineage: LineageId,
        scope: &NamingReadScope,
    ) -> Option<(PersistentNameHolder, PersistentNameIdentity)> {
        let holder = match *self.lineage_holders.get_in(&lineage, scope)? {
            LineageHolder::Entity(entity) => PersistentNameHolder::Entity(entity),
            LineageHolder::Relation(relation) => PersistentNameHolder::Relation(relation),
        };
        let (_, name) = subject_and_name(self.name_of_lineage(lineage, scope)?)?;
        Some((holder, name))
    }

    pub(crate) fn live_holders(
        &self,
    ) -> Result<BTreeMap<PersistentNameIdentity, PersistentNameHolder>, Cause> {
        let mut holders = BTreeMap::new();
        self.names
            .scan_merged(&[self.names.table()], |key, entry| {
                if let Some((_, name)) = subject_and_name(key)
                    && let Some(holder) = entry
                        .current_live()
                        .and_then(|live| holder_of_target(&live.target))
                {
                    holders.insert(name, holder);
                }
                Ok(true)
            })?;
        Ok(holders)
    }

    pub(crate) fn audit_commit_basis(&self) -> Result<AuthoritativeLineageBasis, Cause> {
        let mut basis = AuthoritativeLineageBasis {
            live_holders: self.live_holders()?,
            ..AuthoritativeLineageBasis::default()
        };
        self.names
            .scan_merged(&[self.names.table()], |key, entry| {
                if let Some((_, name)) = subject_and_name(key)
                    && let Some(retired) = retired_lineage(key, &entry)
                {
                    basis.retired_lineages.insert(name, retired);
                }
                Ok(true)
            })?;
        self.holder_lineages
            .scan_merged(&[self.holder_lineages.table()], |holder, epochs| {
                if let Some((_, lineage, None)) = epochs.latest() {
                    let holder = match holder {
                        LineageHolder::Entity(id) => PersistentNameHolder::Entity(id),
                        LineageHolder::Relation(id) => PersistentNameHolder::Relation(id),
                    };
                    basis.live_lineages.insert(holder, lineage);
                }
                Ok(true)
            })?;
        Ok(basis)
    }
}

fn retired_lineage(
    key: PersistentNameKey,
    entry: &NameEntry,
) -> Option<(LineageSubject, LineageId)> {
    let (ended_at, NameState::Ended) = entry.last_state() else {
        return None;
    };
    let (subject, _) = subject_and_name(key)?;
    Some((subject, entry.lineage_ended_at(ended_at)?))
}
