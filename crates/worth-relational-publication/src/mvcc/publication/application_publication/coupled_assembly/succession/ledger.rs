//! One chain's creates by scoped key (#119 M-ID, M-N4, N2, N3).
//!
//! Each key a round of the chain created keeps its identity for the whole
//! candidate: `Bound` while the live round creates it, `Dormant` once a later
//! round's retraction superseded it. A round that creates the key again,
//! adjacent or not, reuses the identity; a key no round creates any more is
//! cancelled at completion. Keys and identities stay in bijection: a slot is
//! never given to a different key inside one candidate.
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_source::AllocationSource;
use contracts::identity::data::primitives::PartitionId;
use contracts::identity::data::record_ids::{EntityId, RelationId};
use contracts::transactions::data::primitives::RecordRef;
use std::collections::{BTreeMap, BTreeSet};
use worth_relational_contracts::internal_port as contracts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reservation {
    Bound,
    Dormant,
}

/// One created record: its source, identity and key text.
pub struct ChainCreate {
    pub source: AllocationSource,
    pub record: RecordRef,
    pub text: String,
}

#[derive(Default)]
pub struct ChainCreateLedger {
    entries: BTreeMap<AllocationSource, (RecordRef, Reservation)>,
    by_record: BTreeMap<RecordRef, AllocationSource>,
    /// Key text to partition, entity (`false`) and relation keys apart.
    partitions: BTreeMap<(bool, String), PartitionId>,
}

impl ChainCreateLedger {
    /// Bind the records the live round created. O(creates · log n).
    pub fn bind_round(&mut self, creates: &[ChainCreate]) {
        for create in creates {
            let (relation, partition) = match &create.source {
                AllocationSource::Entity { partition_id, .. } => (false, *partition_id),
                AllocationSource::Relation { partition_id, .. } => (true, *partition_id),
                AllocationSource::Replacement(_) => continue,
            };
            self.partitions
                .insert((relation, create.text.clone()), partition);
            self.by_record
                .insert(create.record.clone(), create.source.clone());
            self.entries.insert(
                create.source.clone(),
                (create.record.clone(), Reservation::Bound),
            );
        }
    }

    /// Withdraw every bound create: the live round is superseded. Returns the
    /// entities and relations whose bindings leave the candidate.
    pub fn withdraw(&mut self) -> (BTreeSet<EntityId>, BTreeSet<RelationId>) {
        let (mut entities, mut relations) = (BTreeSet::new(), BTreeSet::new());
        for (record, reservation) in self.entries.values_mut() {
            if *reservation == Reservation::Bound {
                *reservation = Reservation::Dormant;
                match record {
                    RecordRef::Entity(id) => entities.insert(*id),
                    RecordRef::Relation(id) => relations.insert(*id),
                };
            }
        }
        (entities, relations)
    }

    /// Every identity a round may reuse, by its source.
    pub fn prebound(&self) -> BTreeMap<AllocationSource, RecordRef> {
        self.entries
            .iter()
            .map(|(source, (record, _))| (source.clone(), record.clone()))
            .collect()
    }

    /// Whether `record` was created by a superseded round and no live one.
    pub fn is_dormant(&self, record: &RecordRef) -> bool {
        self.by_record
            .get(record)
            .and_then(|source| self.entries.get(source))
            .is_some_and(|(_, reservation)| *reservation == Reservation::Dormant)
    }

    /// The identities no live round creates.
    pub fn dormant(&self) -> impl Iterator<Item = &RecordRef> {
        self.entries
            .values()
            .filter(|(_, reservation)| *reservation == Reservation::Dormant)
            .map(|(record, _)| record)
    }

    /// The partition the chain gave key `text`, entity or relation.
    pub fn partition(&self, relation: bool, text: &str) -> Option<PartitionId> {
        self.partitions.get(&(relation, text.to_owned())).copied()
    }

    /// Whether any round of the chain used scoped key `text`.
    pub fn knows_text(&self, text: &str) -> bool {
        self.partitions.contains_key(&(false, text.to_owned()))
            || self.partitions.contains_key(&(true, text.to_owned()))
    }
}
