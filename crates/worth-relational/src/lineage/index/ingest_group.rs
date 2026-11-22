//! Group lineage ingestion into the derived name index.
//!
//! `Born` starts a group name and `Revived` makes a retired one live again;
//! `Retired` ends it. `SplitFrom` and `MergedInto` record forward edges at
//! their commit. Resolution follows a group's edges only from the commit at
//! which it ended, so a fragment split off a kept group never answers for
//! the group that kept its name.

use std::collections::BTreeSet;

use crate::aggregate_identity::AggregateGroupLineageEvent as Event;
use crate::lineage::data::{PersistentNameKey, PersistentNameTarget};

use super::journal::IngestionJournal;
use super::{
    LiveName, NameEntry, NameState, PersistentNameBranchIndex, PersistentNameCommittedBatch,
    PersistentNameIndexDenial,
};

fn invalid(name: PersistentNameKey, detail: &'static str) -> PersistentNameIndexDenial {
    PersistentNameIndexDenial::InvalidTransition { name, detail }
}

fn live_group(group: worth_foundational::facade::AggregateIdentity) -> LiveName {
    LiveName {
        target: PersistentNameTarget::Group { group },
        carriers: BTreeSet::new(),
    }
}

impl PersistentNameBranchIndex {
    pub(super) fn ingest_group_events(
        &mut self,
        batch: &PersistentNameCommittedBatch,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let commit = batch.commit;
        for event in &batch.group_events {
            match *event {
                Event::Born { group } => {
                    let entry = NameEntry::born(commit, None, live_group(group));
                    self.insert_entry(PersistentNameKey::Group(group), entry, journal)?;
                }
                Event::Revived { group } => {
                    let key = PersistentNameKey::Group(group);
                    match self.names.get(&key).map(NameEntry::current) {
                        Some(NameState::Ended) => self
                            .entry_mut(key, journal)
                            .set_state(commit, NameState::Live(live_group(group))),
                        Some(NameState::Live(_)) => {
                            return Err(invalid(key, "a live group cannot be revived"));
                        }
                        None => return Err(invalid(key, "a revived group was never born")),
                    }
                }
                Event::SplitFrom { .. } | Event::MergedInto { .. } | Event::Retired { .. } => {}
            }
        }
        for event in &batch.group_events {
            if let Event::Retired { group } = *event {
                let key = PersistentNameKey::Group(group);
                if self
                    .names
                    .get(&key)
                    .and_then(NameEntry::current_live)
                    .is_none()
                {
                    return Err(invalid(key, "only a live group can retire"));
                }
                self.entry_mut(key, journal)
                    .set_state(commit, NameState::Ended);
            }
        }
        for event in &batch.group_events {
            let (from, to) = match *event {
                Event::SplitFrom { fragment, source } => (source, fragment),
                Event::MergedInto { absorbed, keeper } => (absorbed, keeper),
                Event::Born { .. } | Event::Revived { .. } | Event::Retired { .. } => continue,
            };
            let (from, to) = (PersistentNameKey::Group(from), PersistentNameKey::Group(to));
            if self
                .names
                .get(&to)
                .and_then(NameEntry::current_live)
                .is_none()
            {
                return Err(invalid(to, "a group lineage edge must reach a live group"));
            }
            if !self.names.contains_key(&from) {
                return Err(invalid(
                    from,
                    "a group lineage edge leaves an unknown group",
                ));
            }
            self.entry_mut(from, journal).add_edge(commit, to);
        }
        Ok(())
    }
}
