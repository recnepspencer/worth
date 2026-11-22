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
                    journal.save_lineage_holder(lineage, None);
                    self.lineage_holders.insert(lineage, holder);
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
            if event.kind() == LineageEventKind::Create {
                self.ingest_create(batch.commit, event, journal)?;
            }
        }
        for event in &batch.lineage_events {
            match event.kind() {
                LineageEventKind::Create => {}
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
            let subject_matches = matches!(
                (event.subject(), birth.subject_kind()),
                (LineageSubject::Entity, PersistentNameSubjectKind::Entity)
                    | (
                        LineageSubject::Relation,
                        PersistentNameSubjectKind::Relation
                    )
            );
            if !subject_matches {
                return Err(invalid(key, "birth subject differs from its event subject"));
            }
            let live = LiveName {
                target: target_for(key, birth.identity(), holder, lineage)?,
                carriers: BTreeSet::new(),
            };
            let side = Some(birth.parts().canonical_discriminator().to_vec());
            self.insert_entry(key, NameEntry::born(commit, side, live), journal)?;
            self.bind_lineage_name(lineage, key, journal);
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
            self.bind_lineage_name(lineage, key, journal);
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
        for source in event.sources() {
            let Some(&key) = self.lineage_names.get(source) else {
                continue;
            };
            if successors.binary_search(&key).is_ok() {
                continue;
            }
            let held_by_source = self
                .names
                .get(&key)
                .and_then(NameEntry::current_live)
                .is_some_and(|live| live_lineage(&live.target) == Some(*source));
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

    fn bind_lineage_name(
        &mut self,
        lineage: LineageId,
        key: PersistentNameKey,
        journal: &mut IngestionJournal,
    ) {
        let prior = self.lineage_names.insert(lineage, key);
        journal.save_lineage_name(lineage, prior);
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
