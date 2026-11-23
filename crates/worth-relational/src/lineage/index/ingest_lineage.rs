//! Entity and relation lineage ingestion into the derived name index.
//!
//! A `Create` event binds a newly born name (or a kept merge keeper's name)
//! to its created lineage's record. A `Replace`, `Split`, or `Merge` event
//! ends every source name that does not continue among its targets and
//! records forward edges from it to every named target lineage; a kept name
//! continues. A `Retire` event ends its source name. An unpostured `Replace`
//! targets an unnamed lineage, so Phase 2 carries no name across it and the
//! source name ends there with no named successor.

use std::collections::BTreeSet;

use crate::history::data::CommitId;
use crate::identity::data::LineageId;
use crate::lineage::data::{
    LineageEventKind, LineageEventRecord, LineageSubject, PersistentNameIdentity,
    PersistentNameKey, PersistentNameSubjectKind, PersistentNameTarget,
};

use super::journal::IngestionJournal;
use super::{
    LineageHolder, LiveName, NameEntry, NameState, PersistentNameBranchIndex,
    PersistentNameCommittedBatch, PersistentNameIndexDenial,
};

const fn subject_kind(subject: LineageSubject) -> PersistentNameSubjectKind {
    match subject {
        LineageSubject::Entity => PersistentNameSubjectKind::Entity,
        LineageSubject::Relation => PersistentNameSubjectKind::Relation,
    }
}

fn subject_key(subject: LineageSubject, name: PersistentNameIdentity) -> PersistentNameKey {
    match subject {
        LineageSubject::Entity => PersistentNameKey::Entity(name),
        LineageSubject::Relation => PersistentNameKey::Relation(name),
    }
}

fn invalid(name: PersistentNameKey, detail: &'static str) -> PersistentNameIndexDenial {
    PersistentNameIndexDenial::InvalidTransition { name, detail }
}

impl PersistentNameBranchIndex {
    pub(super) fn ingest_lineage_events(
        &mut self,
        batch: &PersistentNameCommittedBatch,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let holders = batch
            .entity_holders
            .iter()
            .map(|(lineage, entity)| (*lineage, LineageHolder::Entity(*entity)))
            .chain(
                batch
                    .relation_holders
                    .iter()
                    .map(|(lineage, relation)| (*lineage, LineageHolder::Relation(*relation))),
            );
        for (lineage, holder) in holders {
            match self.lineage_holders.get(&lineage) {
                Some(bound) if *bound == holder => {}
                Some(_) => return Err(PersistentNameIndexDenial::ConflictingHolder { lineage }),
                None => {
                    self.lineage_holders.insert(lineage, holder);
                    self.holder_lineages
                        .entry(holder)
                        .or_default()
                        .push((batch.commit, lineage));
                    journal.bound_lineage_holder(lineage);
                }
            }
        }
        for event in &batch.lineage_events {
            if event.commit().commit_id != batch.commit || event.branch_id() != self.branch_id() {
                return Err(PersistentNameIndexDenial::ForeignEvent {
                    expected_branch: self.branch_id().clone(),
                    expected_commit: batch.commit,
                    event_id: event.event_id(),
                });
            }
        }
        // Bindings first, so transitions in the same commit see their targets.
        for event in &batch.lineage_events {
            match event.kind() {
                LineageEventKind::Create => self.ingest_create(batch.commit, event, journal)?,
                LineageEventKind::Restore => self.ingest_restore(batch.commit, event, journal)?,
                LineageEventKind::Retire
                | LineageEventKind::Replace
                | LineageEventKind::Split
                | LineageEventKind::Merge => {}
            }
        }
        for event in &batch.lineage_events {
            match event.kind() {
                LineageEventKind::Create | LineageEventKind::Restore => {}
                LineageEventKind::Retire
                | LineageEventKind::Replace
                | LineageEventKind::Split
                | LineageEventKind::Merge => {
                    self.ingest_transition(batch.commit, event, journal)?
                }
            }
        }
        Ok(())
    }

    fn ingest_restore(
        &mut self,
        commit: CommitId,
        event: &LineageEventRecord,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let fallback_key = event
            .kept_name()
            .map(|name| subject_key(event.subject(), name))
            .or_else(|| {
                event
                    .sources()
                    .first()
                    .and_then(|src| self.lineage_names.get(src).copied())
            })
            .unwrap_or_else(|| {
                subject_key(
                    event.subject(),
                    PersistentNameIdentity::from_minted_digest([0u8; 32]),
                )
            });
        let Some(name) = event.kept_name() else {
            return Err(invalid(fallback_key, "a Restore must carry kept_name"));
        };
        let key = subject_key(event.subject(), name);
        if event.name_birth().is_some() {
            return Err(invalid(key, "a Restore cannot carry a name_birth"));
        }
        let (&[source], &[target]) = (event.sources(), event.targets()) else {
            return Err(invalid(
                key,
                "a Restore must have exactly one source and one target lineage",
            ));
        };
        if source == target {
            return Err(invalid(
                key,
                "a Restore target lineage must differ from its retired source lineage",
            ));
        }
        let Some(entry) = self.names.get(&key) else {
            let opposite = match event.subject() {
                LineageSubject::Entity => PersistentNameKey::Relation(name),
                LineageSubject::Relation => PersistentNameKey::Entity(name),
            };
            if self.names.contains_key(&opposite) {
                return Err(invalid(
                    key,
                    "a restored name's subject differs from its event subject",
                ));
            }
            return Err(invalid(
                key,
                "a restored name does not exist in this branch",
            ));
        };
        match entry.last_state() {
            (_, NameState::Live(_)) => {
                return Err(invalid(key, "a live name cannot be restored"));
            }
            (ended_at, NameState::Ended) if ended_at >= commit => {
                return Err(invalid(
                    key,
                    "a restored name must have retired at an earlier commit",
                ));
            }
            (_, NameState::Ended) => {}
        }
        if self.lineage_names.get(&source) != Some(&key) {
            return Err(invalid(
                key,
                "a Restore source lineage does not match the restored name",
            ));
        }
        if entry.last_live_lineage() != Some(source) {
            return Err(invalid(
                key,
                "a Restore source lineage is not the name's most recent retired lineage",
            ));
        }
        let holder = *self
            .lineage_holders
            .get(&target)
            .ok_or(PersistentNameIndexDenial::MissingHolder { lineage: target })?;
        let live = LiveName {
            target: target_for(key, name, holder, target)?,
            carriers: BTreeSet::new(),
        };
        self.bind_lineage_name(target, key, journal)?;
        self.entry_mut(key, journal)
            .set_state(commit, NameState::Live(live));
        Ok(())
    }

    fn ingest_create(
        &mut self,
        commit: CommitId,
        event: &LineageEventRecord,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let (birth, kept) = (event.name_birth(), event.kept_name());
        let Some(name) = birth.map(|birth| birth.identity()).or(kept) else {
            return Ok(());
        };
        if birth.is_some() && kept.is_some() {
            return Err(invalid(
                subject_key(event.subject(), name),
                "a Create cannot both birth a name and keep another",
            ));
        }
        let &[lineage] = event.targets() else {
            return Err(invalid(
                subject_key(event.subject(), name),
                "a named Create must target exactly one lineage",
            ));
        };
        let holder = *self
            .lineage_holders
            .get(&lineage)
            .ok_or(PersistentNameIndexDenial::MissingHolder { lineage })?;
        if let Some(birth) = birth {
            let key = subject_key(event.subject(), birth.identity());
            if birth.subject_kind() != subject_kind(event.subject()) {
                return Err(invalid(key, "birth subject differs from its event subject"));
            }
            let live = LiveName {
                target: target_for(key, birth.identity(), holder, lineage)?,
                carriers: BTreeSet::new(),
            };
            // A name born without a discriminator has no side, exactly like a group.
            let canonical = birth.parts().canonical_discriminator();
            let side = (!canonical.is_empty()).then(|| canonical.to_vec());
            let scheme = Some(birth.parts().scheme_id().to_owned());
            let entry = NameEntry::born(commit, side, scheme, live);
            self.insert_entry(key, entry, journal)?;
            self.bind_lineage_name(lineage, key, journal)?;
        }
        if let Some(kept) = kept {
            let key = subject_key(event.subject(), kept);
            let Some(previous) = self.names.get(&key).and_then(NameEntry::current_live) else {
                return Err(invalid(
                    key,
                    "a kept merge name must be live to move records",
                ));
            };
            let live = LiveName {
                target: target_for(key, kept, holder, lineage)?,
                carriers: previous.carriers.clone(),
            };
            self.entry_mut(key, journal)
                .set_state(commit, NameState::Live(live));
            self.bind_lineage_name(lineage, key, journal)?;
        }
        Ok(())
    }

    fn ingest_transition(
        &mut self,
        commit: CommitId,
        event: &LineageEventRecord,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let mut successors: Vec<PersistentNameKey> = event
            .targets()
            .iter()
            .filter_map(|lineage| self.lineage_names.get(lineage).copied())
            .collect();
        successors.sort_unstable();
        successors.dedup();
        let subject = subject_kind(event.subject());
        if let Some(foreign) = successors.iter().find(|key| key.subject_kind() != subject) {
            return Err(invalid(
                *foreign,
                "a successor differs from its event subject",
            ));
        }
        for source in event.sources() {
            let Some(&key) = self.lineage_names.get(source) else {
                continue;
            };
            if key.subject_kind() != subject {
                return Err(invalid(key, "a source differs from its event subject"));
            }
            if successors.binary_search(&key).is_ok() {
                continue;
            }
            let held_by_source = self.names.get(&key).is_some_and(|entry| {
                matches!(
                    entry.last_state(),
                    (since, NameState::Live(live))
                        if since < commit && live_lineage(&live.target) == Some(*source)
                )
            });
            if !held_by_source {
                return Err(invalid(
                    key,
                    "an ending source lineage no longer holds its name",
                ));
            }
            let entry = self.entry_mut(key, journal);
            entry.set_state(commit, NameState::Ended);
            for successor in &successors {
                entry.add_edge(commit, *successor);
            }
        }
        Ok(())
    }

    /// Binds a created lineage to the name it carries; a lineage is created
    /// once, so a second binding is corrupt history and refuses.
    fn bind_lineage_name(
        &mut self,
        lineage: LineageId,
        key: PersistentNameKey,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        if self.lineage_names.contains_key(&lineage) {
            return Err(invalid(key, "a lineage cannot be bound to a second name"));
        }
        journal.bound_lineage_name(lineage);
        self.lineage_names.insert(lineage, key);
        Ok(())
    }
}

fn target_for(
    key: PersistentNameKey,
    name: PersistentNameIdentity,
    holder: LineageHolder,
    lineage: LineageId,
) -> Result<PersistentNameTarget, PersistentNameIndexDenial> {
    match (key, holder) {
        (PersistentNameKey::Entity(_), LineageHolder::Entity(entity)) => {
            Ok(PersistentNameTarget::Entity {
                name,
                entity,
                lineage,
            })
        }
        (PersistentNameKey::Relation(_), LineageHolder::Relation(relation)) => {
            Ok(PersistentNameTarget::Relation {
                name,
                relation,
                lineage,
            })
        }
        _ => Err(invalid(
            key,
            "a name's record differs from its subject class",
        )),
    }
}

const fn live_lineage(target: &PersistentNameTarget) -> Option<LineageId> {
    match target {
        PersistentNameTarget::Entity { lineage, .. }
        | PersistentNameTarget::Relation { lineage, .. } => Some(*lineage),
        PersistentNameTarget::Group { .. } => None,
    }
}
