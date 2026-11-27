//! Group lineage ingestion into the derived name index.
//!
//! `Born` starts a group name and `Revived` makes a retired one live again;
//! `Retired` ends a group that was live before the commit. `SplitFrom` and
//! `MergedInto` record forward edges at their commit: a split leaves a group
//! that was live before the commit, a merge leaves a group retired in this
//! commit, and both reach a different group that is live after it.
//! Resolution follows a group's edges only from the commit at which it ended,
//! so a fragment split off a kept group never answers for the group that
//! kept its name.

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameTarget};

use super::journal::IngestionJournal;
use super::{
    Carriers, LiveName, NameEntry, NameOrigin, NameState, PersistentNameBranchIndex,
    PersistentNameCommittedBatch, PersistentNameIndexDenial,
};

fn invalid(group: AggregateIdentity, detail: &'static str) -> PersistentNameIndexDenial {
    PersistentNameIndexDenial::InvalidTransition {
        name: PersistentNameKey::Group(group),
        detail,
    }
}

fn live_group(group: AggregateIdentity) -> LiveName {
    LiveName {
        target: PersistentNameTarget::Group { group },
        carriers: Carriers::new(),
    }
}

impl PersistentNameBranchIndex {
    pub(super) fn ingest_group_events(
        &mut self,
        batch: &PersistentNameCommittedBatch,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let events = &batch.group_events;
        // Pre-commit liveness of every group this batch mentions, read before
        // any of its events changes the index.
        let mut live_before = BTreeSet::new();
        for group in events.iter().flat_map(|event| event_groups(*event)) {
            if self.group_is_live(group)? {
                live_before.insert(group);
            }
        }
        self.ingest_group_births(batch.commit, events, &batch.group_schemes, journal)?;
        let retired = self.ingest_group_retirements(batch.commit, events, &live_before, journal)?;
        self.ingest_group_edges(batch.commit, events, &live_before, &retired, journal)
    }

    fn group_is_live(&self, group: AggregateIdentity) -> Result<bool, PersistentNameIndexDenial> {
        let entry = self.names.get(&PersistentNameKey::Group(group))?;
        Ok(entry.as_deref().and_then(NameEntry::current_live).is_some())
    }

    fn ingest_group_births(
        &mut self,
        commit: CommitId,
        events: &BTreeSet<Event>,
        schemes: &BTreeMap<AggregateIdentity, String>,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        for event in events {
            match *event {
                Event::Born { group } => {
                    let origin = NameOrigin::Group {
                        scheme_id: schemes.get(&group).cloned(),
                    };
                    let entry = NameEntry::born(commit, origin, live_group(group));
                    self.insert_entry(PersistentNameKey::Group(group), commit, entry, journal)?;
                }
                Event::Revived { group } => {
                    let key = PersistentNameKey::Group(group);
                    let ended = self
                        .names
                        .get(&key)?
                        .map(|entry| *entry.current() == NameState::Ended);
                    match ended {
                        Some(true) => self
                            .entry_mut(key, commit, journal)?
                            .set_state(commit, NameState::Live(live_group(group))),
                        Some(false) => {
                            return Err(invalid(group, "a live group cannot be revived"));
                        }
                        None => return Err(invalid(group, "a revived group was never born")),
                    }
                }
                Event::SplitFrom { .. } | Event::MergedInto { .. } | Event::Retired { .. } => {}
            }
        }
        Ok(())
    }

    /// Ends every retired group; returns the groups retired in this commit.
    fn ingest_group_retirements(
        &mut self,
        commit: CommitId,
        events: &BTreeSet<Event>,
        live_before: &BTreeSet<AggregateIdentity>,
        journal: &mut IngestionJournal,
    ) -> Result<BTreeSet<AggregateIdentity>, PersistentNameIndexDenial> {
        let mut retired = BTreeSet::new();
        for event in events {
            let Event::Retired { group } = *event else {
                continue;
            };
            if !live_before.contains(&group) {
                return Err(invalid(
                    group,
                    "only a group live before the commit can retire",
                ));
            }
            let entry = self.names.get(&PersistentNameKey::Group(group))?;
            let members = entry
                .as_deref()
                .and_then(NameEntry::current_live)
                .map(|live| live.carriers.clone())
                .unwrap_or_default();
            self.end_member_tenures(group, &members, commit, journal)?;
            self.entry_mut(PersistentNameKey::Group(group), commit, journal)?
                .set_state(commit, NameState::Ended);
            retired.insert(group);
        }
        Ok(retired)
    }

    fn ingest_group_edges(
        &mut self,
        commit: CommitId,
        events: &BTreeSet<Event>,
        live_before: &BTreeSet<AggregateIdentity>,
        retired: &BTreeSet<AggregateIdentity>,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        for event in events {
            let (from, to) = match *event {
                Event::SplitFrom { fragment, source } => {
                    if !live_before.contains(&source) {
                        return Err(invalid(source, "a split must leave a group live before it"));
                    }
                    (source, fragment)
                }
                Event::MergedInto { absorbed, keeper } => {
                    if !retired.contains(&absorbed) {
                        return Err(invalid(
                            absorbed,
                            "a merge must leave a group retired by it",
                        ));
                    }
                    (absorbed, keeper)
                }
                Event::Born { .. } | Event::Revived { .. } | Event::Retired { .. } => continue,
            };
            if from == to {
                return Err(invalid(from, "a group lineage edge cannot loop to itself"));
            }
            if !self.group_is_live(to)? {
                return Err(invalid(to, "a group lineage edge must reach a live group"));
            }
            self.entry_mut(PersistentNameKey::Group(from), commit, journal)?
                .add_edge(commit, PersistentNameKey::Group(to));
        }
        Ok(())
    }
}

/// Every group an event names.
fn event_groups(event: Event) -> Vec<AggregateIdentity> {
    match event {
        Event::Born { group } | Event::Revived { group } | Event::Retired { group } => vec![group],
        Event::SplitFrom { fragment, source } => vec![fragment, source],
        Event::MergedInto { absorbed, keeper } => vec![absorbed, keeper],
    }
}
